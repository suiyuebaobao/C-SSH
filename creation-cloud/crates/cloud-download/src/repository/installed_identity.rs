//! 以固定父子锁顺序读取发布与版本化正式资产，并在同一事务写最小语义审计。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::repository::map_read_error;

#[derive(Debug, FromRow)]
pub(crate) struct LockedIdentityRelease {
    pub id: Uuid,
    pub version: String,
    pub channel: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct LockedIdentityAsset {
    pub id: Uuid,
    pub release_id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub installed_sha256: Option<String>,
}

pub(crate) const LOCK_IDENTITY_RELEASE_SQL: &str = r#"
    SELECT id, version, channel, status, updated_at
    FROM releases
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) const LOCK_IDENTITY_ASSETS_SQL: &str = r#"
    SELECT id, release_id, platform, architecture, package_kind,
           file_name, byte_size, sha256, installed_sha256
    FROM release_assets
    WHERE release_id = $1
    ORDER BY CASE
        WHEN platform = 'windows' AND architecture = 'x86_64' AND package_kind = 'exe' THEN 1
        WHEN platform = 'windows' AND architecture = 'x86_64' AND package_kind = 'msi' THEN 2
        WHEN platform = 'windows' AND architecture = 'x86_64' AND package_kind = 'zip' THEN 3
        WHEN platform = 'android' AND architecture = 'aarch64' AND package_kind = 'apk' THEN 4
        ELSE 5
    END, id
    FOR UPDATE
"#;

pub(crate) async fn lock_release(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> AppResult<LockedIdentityRelease> {
    sqlx::query_as::<_, LockedIdentityRelease>(LOCK_IDENTITY_RELEASE_SQL)
        .bind(release_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("版本不存在".into()))
}

pub(crate) async fn lock_assets(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> AppResult<Vec<LockedIdentityAsset>> {
    sqlx::query_as::<_, LockedIdentityAsset>(LOCK_IDENTITY_ASSETS_SQL)
        .bind(release_id)
        .fetch_all(&mut **transaction)
        .await
        .map_err(map_read_error)
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    release_id: Uuid,
    details: Value,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'release_asset.installed_identity_batch_recorded', 'release', \
                 $3, 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(release_id.to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存安装身份审计失败".into()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_is_locked_before_independently_ordered_asset_rows() {
        assert!(LOCK_IDENTITY_RELEASE_SQL.contains("FROM releases"));
        assert!(LOCK_IDENTITY_RELEASE_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_IDENTITY_RELEASE_SQL.contains("JOIN release_assets"));
        assert!(LOCK_IDENTITY_ASSETS_SQL.contains("FROM release_assets"));
        assert!(LOCK_IDENTITY_ASSETS_SQL.contains("FOR UPDATE"));
        for package_kind in ["exe", "msi", "zip", "apk"] {
            assert!(LOCK_IDENTITY_ASSETS_SQL.contains(package_kind));
        }
    }
}
