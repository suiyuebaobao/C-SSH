//! 按版本标识锁定父版本行，为资产写入提供统一的第一层锁。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{Release, model::ReleaseRow, repository::map_read_error};

pub(crate) const LOCK_RELEASE_SQL: &str = r#"
    SELECT id, version, channel, status, title_zh, title_en,
           notes_zh, notes_en, published_at, created_at, updated_at
    FROM releases
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) async fn execute(connection: &mut PgConnection, id: Uuid) -> AppResult<Release> {
    let row = sqlx::query_as::<_, ReleaseRow>(LOCK_RELEASE_SQL)
        .bind(id)
        .fetch_optional(connection)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("版本不存在".into()))?;
    Release::try_from(row)
}

#[cfg(test)]
mod tests {
    use super::LOCK_RELEASE_SQL;

    #[test]
    fn parent_release_lock_is_explicit() {
        assert!(LOCK_RELEASE_SQL.contains("FROM releases"));
        assert!(LOCK_RELEASE_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_RELEASE_SQL.contains("JOIN release_assets"));
    }
}
