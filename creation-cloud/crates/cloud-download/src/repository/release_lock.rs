//! 独立锁定下载来源所属的父版本，固定层级写入的第一把数据库锁。

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::repository::map_read_error;

pub(crate) const LOCK_RELEASE_SQL: &str = "SELECT status FROM releases WHERE id = $1 FOR UPDATE";

pub(crate) async fn execute(connection: &mut PgConnection, release_id: Uuid) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(LOCK_RELEASE_SQL)
        .bind(release_id)
        .fetch_optional(connection)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("版本不存在".into()))
}

#[cfg(test)]
mod tests {
    use super::LOCK_RELEASE_SQL;

    #[test]
    fn parent_release_lock_has_no_child_join() {
        assert!(LOCK_RELEASE_SQL.contains("FROM releases"));
        assert!(LOCK_RELEASE_SQL.contains("FOR UPDATE"));
        assert!(!LOCK_RELEASE_SQL.contains("JOIN"));
    }
}
