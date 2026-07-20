//! 在父版本已经锁定后锁定单个资产，并返回完整资产身份供用例复核。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ReleaseAsset, model::AssetRow, repository::map_read_error};

pub(crate) const LOCK_ASSET_SQL: &str = r#"
    SELECT id, release_id, platform, architecture, package_kind,
           file_name, byte_size, sha256, created_at
    FROM release_assets
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<ReleaseAsset> {
    sqlx::query_as::<_, AssetRow>(LOCK_ASSET_SQL)
        .bind(id)
        .fetch_optional(connection)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("资产不存在".into()))
}

#[cfg(test)]
mod tests {
    use super::LOCK_ASSET_SQL;

    #[test]
    fn child_asset_lock_is_independent_from_parent_lock() {
        assert!(LOCK_ASSET_SQL.contains("FROM release_assets"));
        assert!(LOCK_ASSET_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_ASSET_SQL.contains("JOIN releases"));
    }
}
