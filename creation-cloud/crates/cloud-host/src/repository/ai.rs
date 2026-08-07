//! 封装 AI provider 账号不透明密文的当前行锁定与版本写入。

use cloud_domain::AppResult;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{actor::DeviceActor, validation::ValidatedAiPayload};

use super::{DbTransaction, storage};

#[derive(Clone, Debug, FromRow)]
pub(super) struct AiRow {
    pub ciphertext: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub envelope_metadata: Option<Value>,
    pub revision: i64,
    pub is_deleted: bool,
}

#[derive(Clone, Debug)]
pub(super) struct AiWriteValue {
    pub ciphertext: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub envelope_metadata: Option<Value>,
    pub deleted: bool,
}

impl AiWriteValue {
    pub(super) fn from_payload(payload: &ValidatedAiPayload) -> Self {
        Self {
            ciphertext: Some(payload.ciphertext.clone()),
            nonce: Some(payload.nonce.clone()),
            envelope_metadata: Some(payload.envelope_metadata.clone()),
            deleted: false,
        }
    }

    pub(super) const fn tombstone() -> Self {
        Self {
            ciphertext: None,
            nonce: None,
            envelope_metadata: None,
            deleted: true,
        }
    }
}

pub(super) async fn lock_current(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    resource_id: Uuid,
) -> AppResult<Option<AiRow>> {
    sqlx::query_as::<_, AiRow>(
        "SELECT ciphertext, nonce, envelope_metadata, revision, is_deleted
         FROM cloud_ai_provider_configs
         WHERE account_id = $1 AND id = $2
         FOR UPDATE",
    )
    .bind(account_id)
    .bind(resource_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}

pub(super) fn same_value(current: &AiRow, value: &AiWriteValue) -> bool {
    current.ciphertext == value.ciphertext
        && current.nonce == value.nonce
        && current.envelope_metadata == value.envelope_metadata
        && current.is_deleted == value.deleted
}

pub(super) async fn write(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    resource_id: Uuid,
    revision: i64,
    value: AiWriteValue,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO cloud_ai_provider_configs
             (account_id, id, ciphertext, nonce, envelope_metadata,
              source_device_id, revision, is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         ON CONFLICT (account_id, id) DO UPDATE SET
             ciphertext = EXCLUDED.ciphertext,
             nonce = EXCLUDED.nonce,
             envelope_metadata = EXCLUDED.envelope_metadata,
             source_device_id = EXCLUDED.source_device_id,
             revision = EXCLUDED.revision,
             is_deleted = EXCLUDED.is_deleted,
             updated_at = now()",
    )
    .bind(actor.account_id())
    .bind(resource_id)
    .bind(&value.ciphertext)
    .bind(&value.nonce)
    .bind(&value.envelope_metadata)
    .bind(actor.device_id())
    .bind(revision)
    .bind(value.deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;

    sqlx::query(
        "INSERT INTO cloud_ai_provider_config_versions
             (account_id, resource_id, revision, ciphertext, nonce,
              envelope_metadata, source_device_id, is_deleted)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(actor.account_id())
    .bind(resource_id)
    .bind(revision)
    .bind(value.ciphertext)
    .bind(value.nonce)
    .bind(value.envelope_metadata)
    .bind(actor.device_id())
    .bind(value.deleted)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}
