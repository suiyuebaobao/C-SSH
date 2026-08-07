//! 渲染保险库信封与包装参数元数据，丢弃密文、nonce 和 salt 后再进模板。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};
use cloud_vault::{VaultEnvelope, VaultKeyWrapper};
use uuid::Uuid;

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

struct VaultEnvelopeView {
    id: Uuid,
    envelope_key: Uuid,
    revision: i64,
    schema_version: i32,
    key_version: i32,
    cipher_suite: String,
    kdf_algorithm: String,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    updated_at: String,
}

impl From<VaultEnvelope> for VaultEnvelopeView {
    fn from(envelope: VaultEnvelope) -> Self {
        Self {
            id: envelope.id,
            envelope_key: envelope.envelope_key,
            revision: envelope.revision,
            schema_version: envelope.schema_version,
            key_version: envelope.key_version,
            cipher_suite: envelope.cipher_suite,
            kdf_algorithm: envelope.kdf.algorithm,
            memory_kib: envelope.kdf.memory_kib,
            iterations: envelope.kdf.iterations,
            parallelism: envelope.kdf.parallelism,
            updated_at: envelope.updated_at.to_rfc3339(),
        }
    }
}

struct VaultWrapperView {
    id: Uuid,
    wrapper_key: Uuid,
    revision: i64,
    schema_version: i32,
    key_version: i32,
    cipher_suite: String,
    kdf_algorithm: String,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
    updated_at: String,
}

impl From<VaultKeyWrapper> for VaultWrapperView {
    fn from(wrapper: VaultKeyWrapper) -> Self {
        Self {
            id: wrapper.id,
            wrapper_key: wrapper.wrapper_key,
            revision: wrapper.revision,
            schema_version: wrapper.schema_version,
            key_version: wrapper.key_version,
            cipher_suite: wrapper.cipher_suite,
            kdf_algorithm: wrapper.kdf.algorithm,
            memory_kib: wrapper.kdf.memory_kib,
            iterations: wrapper.kdf.iterations,
            parallelism: wrapper.kdf.parallelism,
            updated_at: wrapper.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Template)]
#[template(path = "console-vault.html")]
struct VaultTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    envelopes: Vec<VaultEnvelopeView>,
    total: i64,
    wrappers: Vec<VaultWrapperView>,
    wrapper_total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let (envelopes, wrappers) = tokio::try_join!(
        state.vault().list(session.account_id, common::first_page()),
        state.vault().list_wrappers(&session, common::first_page()),
    )?;
    let locale = query.locale();
    common::render(&VaultTemplate {
        view: common::view(PageId::Vault, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        envelopes: envelopes
            .items
            .into_iter()
            .map(VaultEnvelopeView::from)
            .collect(),
        total: envelopes.total,
        wrappers: wrappers
            .items
            .into_iter()
            .map(VaultWrapperView::from)
            .collect(),
        wrapper_total: wrappers.total,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use cloud_site::Locale;
    use cloud_vault::KdfMetadata;

    use super::*;

    #[test]
    fn template_never_receives_vault_or_wrapper_secrets() {
        let envelope = VaultEnvelope {
            id: Uuid::from_u128(1),
            envelope_key: Uuid::from_u128(2),
            revision: 7,
            schema_version: 3,
            key_version: 4,
            cipher_suite: "xchacha20-poly1305".to_owned(),
            kdf: KdfMetadata {
                algorithm: "argon2id".to_owned(),
                salt: "vault-salt-must-not-render".to_owned(),
                memory_kib: 65_536,
                iterations: 3,
                parallelism: 2,
            },
            nonce: "vault-nonce-must-not-render".to_owned(),
            ciphertext: "vault-ciphertext-must-not-render".to_owned(),
            created_at: Utc.with_ymd_and_hms(2026, 7, 21, 8, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 7, 21, 9, 0, 0).unwrap(),
        };
        let wrapper = VaultKeyWrapper {
            id: Uuid::from_u128(3),
            wrapper_key: Uuid::from_u128(4),
            revision: 8,
            schema_version: 3,
            key_version: 5,
            cipher_suite: "aes-256-gcm".to_owned(),
            kdf: KdfMetadata {
                algorithm: "argon2id".to_owned(),
                salt: "wrapper-salt-must-not-render".to_owned(),
                memory_kib: 131_072,
                iterations: 4,
                parallelism: 1,
            },
            nonce: "wrapper-nonce-must-not-render".to_owned(),
            wrapped_master_key: "wrapped-master-key-must-not-render".to_owned(),
            created_at: Utc.with_ymd_and_hms(2026, 7, 21, 8, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 7, 21, 9, 0, 0).unwrap(),
        };

        let rendered = VaultTemplate {
            view: common::view(PageId::Vault, Locale::ZhCn),
            seo: common::seo(),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            envelopes: vec![VaultEnvelopeView::from(envelope)],
            total: 1,
            wrappers: vec![VaultWrapperView::from(wrapper)],
            wrapper_total: 1,
        }
        .render()
        .expect("保险库控制台模板应可渲染");

        assert!(rendered.contains("xchacha20-poly1305"));
        assert!(rendered.contains("aes-256-gcm"));
        assert!(rendered.contains("revision 7"));
        assert!(rendered.contains("revision 8"));
        assert!(!rendered.contains("vault-salt-must-not-render"));
        assert!(!rendered.contains("vault-nonce-must-not-render"));
        assert!(!rendered.contains("vault-ciphertext-must-not-render"));
        assert!(!rendered.contains("wrapper-salt-must-not-render"));
        assert!(!rendered.contains("wrapper-nonce-must-not-render"));
        assert!(!rendered.contains("wrapped-master-key-must-not-render"));
    }
}
