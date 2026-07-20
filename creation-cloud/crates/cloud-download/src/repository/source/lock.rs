//! 在父版本和资产都已锁定后锁定单个来源，并返回来源身份供用例复核。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ReleaseSource, model::SourceRow, repository::map_read_error};

pub(crate) const LOCK_SOURCE_SQL: &str = r#"
    SELECT id, asset_id, source_kind, provider_name, local_path,
           external_url, sort_order, enabled, created_at, updated_at
    FROM release_sources
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) async fn execute(
    connection: &mut PgConnection,
    source_id: Uuid,
) -> AppResult<ReleaseSource> {
    let row = sqlx::query_as::<_, SourceRow>(LOCK_SOURCE_SQL)
        .bind(source_id)
        .fetch_optional(connection)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("下载来源不存在".into()))?;
    ReleaseSource::try_from(row)
}

#[cfg(test)]
mod tests {
    use super::LOCK_SOURCE_SQL;

    #[test]
    fn source_lock_has_no_parent_join() {
        assert!(LOCK_SOURCE_SQL.contains("FROM release_sources"));
        assert!(LOCK_SOURCE_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_SOURCE_SQL.contains("JOIN"));
    }
}
