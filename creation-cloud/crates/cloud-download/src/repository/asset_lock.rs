//! 在父版本已经锁定后独立锁定资产行，不依赖联表锁的隐式顺序。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{model::LockedAssetRecord, repository::map_read_error};

pub(crate) const LOCK_ASSET_SQL: &str = r#"
    SELECT release_id, byte_size, sha256
    FROM release_assets
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) async fn execute(
    connection: &mut PgConnection,
    asset_id: Uuid,
) -> AppResult<LockedAssetRecord> {
    sqlx::query_as::<_, LockedAssetRecord>(LOCK_ASSET_SQL)
        .bind(asset_id)
        .fetch_optional(connection)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("资产不存在".into()))
}

#[cfg(test)]
mod tests {
    use super::LOCK_ASSET_SQL;

    #[test]
    fn child_asset_lock_has_no_parent_join() {
        assert!(LOCK_ASSET_SQL.contains("FROM release_assets"));
        assert!(LOCK_ASSET_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_ASSET_SQL.contains("JOIN"));
    }
}
