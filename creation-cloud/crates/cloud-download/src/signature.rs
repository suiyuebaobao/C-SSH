//! updater signature 只属于 Windows EXE/MSI/ZIP；其它资产一律拒绝。

use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::Service;

pub(crate) const MAX_SIGNATURE_BYTES: usize = 8192;

pub(crate) fn validate(value: &str) -> AppResult<String> {
    let value = value.trim();
    if !(16..=MAX_SIGNATURE_BYTES).contains(&value.len())
        || !value.is_ascii()
        || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation("updater signature 格式无效".into()));
    }
    Ok(value.to_owned())
}

pub(crate) fn validate_for_asset(
    platform: &str,
    package_kind: &str,
    value: Option<&str>,
) -> AppResult<Option<String>> {
    match value {
        Some(value) if accepts_asset(platform, package_kind) => validate(value).map(Some),
        Some(_) => Err(AppError::Validation(
            "updater signature 只允许 Windows EXE/MSI/ZIP 资产使用".into(),
        )),
        None => Ok(None),
    }
}

fn accepts_asset(platform: &str, package_kind: &str) -> bool {
    platform.trim().eq_ignore_ascii_case("windows")
        && matches!(
            package_kind.trim().to_ascii_lowercase().as_str(),
            "exe" | "msi" | "zip"
        )
}

impl Service {
    pub async fn set_asset_updater_signature(
        &self,
        actor: &AdminActor,
        asset_id: Uuid,
        signature: &str,
    ) -> AppResult<()> {
        let actor_id = require_actor(actor)?;
        if asset_id.is_nil() {
            return Err(AppError::Validation("资产标识无效".into()));
        }
        let signature = validate(signature)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| AppError::Storage("签名事务启动失败".into()))?;
        set_in_transaction(&mut transaction, asset_id, &signature).await?;
        audit(&mut transaction, actor_id, asset_id).await?;
        transaction
            .commit()
            .await
            .map_err(|_| AppError::Storage("签名事务提交失败".into()))?;
        mark_semantic_audit_recorded();
        Ok(())
    }
}

pub(crate) async fn set_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    asset_id: Uuid,
    signature: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE release_assets AS asset SET updater_signature = $2 \
         FROM releases AS release WHERE asset.id = $1 AND release.id = asset.release_id \
           AND asset.platform = 'windows' \
           AND asset.package_kind IN ('exe', 'msi', 'zip') \
           AND (release.status IN ('draft', 'validating') \
             OR (release.status = 'published' \
                 AND asset.updater_signature IS NULL))",
    )
    .bind(asset_id)
    .bind(signature)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存 updater signature 失败".into()))?;
    if result.rows_affected() != 1 {
        return Err(AppError::Conflict(
            "资产不存在、签名已存在或当前状态不能写入 updater signature".into(),
        ));
    }
    Ok(())
}

async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    asset_id: Uuid,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'release_asset.signature_uploaded', 'release_asset', $3, \
                 'success', $4, jsonb_build_object('asset_id', $5::uuid))",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(asset_id.to_string())
    .bind(request_id)
    .bind(asset_id)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存 updater signature 审计失败".into()))?;
    Ok(())
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let actor_id = actor.account_id();
    if actor_id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".into()))
    } else {
        Ok(actor_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_bounded_ascii_without_controls() {
        assert!(validate(&"a".repeat(64)).is_ok());
        assert!(validate("short").is_err());
        assert!(validate(&format!("{}\n", "a".repeat(64))).is_ok());
        assert!(validate(&"中".repeat(64)).is_err());
    }

    #[test]
    fn published_backfill_is_narrowed_to_empty_windows_update_metadata() {
        let source = include_str!("signature.rs");
        for contract in [
            "release.status = 'published'",
            "asset.platform = 'windows'",
            "asset.package_kind IN ('exe', 'msi', 'zip')",
            "asset.updater_signature IS NULL",
        ] {
            assert!(source.contains(contract), "签名回填缺少契约：{contract}");
        }
    }

    #[test]
    fn updater_signature_scope_accepts_only_windows_update_assets() {
        let signature = "a".repeat(64);
        for package_kind in ["exe", "msi", "zip"] {
            assert!(
                validate_for_asset(" Windows ", package_kind, Some(&signature)).is_ok(),
                "Windows {package_kind} 应允许签名"
            );
        }
        assert!(validate_for_asset("android", "apk", Some(&signature)).is_err());
        assert!(validate_for_asset("linux", "appimage", Some(&signature)).is_err());
        assert_eq!(validate_for_asset("android", "apk", None).unwrap(), None);
    }
}
