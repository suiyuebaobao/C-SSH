//! 以稳定游标锁定并删除一批已过期会话。
//! `SKIP LOCKED` 允许多个维护实例互不等待，实际计数来自 `RETURNING`。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::{Executor, PgConnection, Postgres};
use uuid::Uuid;

use crate::model::expired_session_cleanup::ExpiredSessionCleanup;

use super::error;

pub(crate) const DELETE_EXPIRED_SESSIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT id
    FROM sessions
    WHERE expires_at <= $1
    ORDER BY expires_at ASC, id ASC
    FOR UPDATE SKIP LOCKED
    LIMIT $2
)
DELETE FROM sessions AS expired
USING candidates
WHERE expired.id = candidates.id
RETURNING expired.id
"#;

pub(crate) async fn delete_batch(pool: &PgPool, cleanup: &ExpiredSessionCleanup) -> AppResult<u64> {
    delete_batch_with_executor(pool, cleanup).await
}

pub(crate) async fn delete_batch_on_connection(
    connection: &mut PgConnection,
    cleanup: &ExpiredSessionCleanup,
) -> AppResult<u64> {
    delete_batch_with_executor(&mut *connection, cleanup).await
}

async fn delete_batch_with_executor<'executor, E>(
    executor: E,
    cleanup: &ExpiredSessionCleanup,
) -> AppResult<u64>
where
    E: Executor<'executor, Database = Postgres>,
{
    let deleted = sqlx::query_scalar::<_, Uuid>(DELETE_EXPIRED_SESSIONS_SQL)
        .bind(cleanup.delete_before())
        .bind(cleanup.batch_size())
        .fetch_all(executor)
        .await
        .map_err(error::storage)?;
    Ok(deleted.len() as u64)
}
