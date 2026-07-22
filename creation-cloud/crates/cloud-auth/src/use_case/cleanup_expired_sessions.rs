//! 校验维护批量边界，并委托仓储删除过期会话。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::PgConnection;

use crate::{model::expired_session_cleanup::ExpiredSessionCleanup, repository};

pub(crate) async fn execute(
    pool: &PgPool,
    delete_before: DateTime<Utc>,
    batch_size: u32,
) -> AppResult<u64> {
    let cleanup = ExpiredSessionCleanup::new(delete_before, batch_size)?;
    repository::cleanup_expired_sessions::delete_batch(pool, &cleanup).await
}

pub(crate) async fn execute_on_connection(
    connection: &mut PgConnection,
    delete_before: DateTime<Utc>,
    batch_size: u32,
) -> AppResult<u64> {
    let cleanup = ExpiredSessionCleanup::new(delete_before, batch_size)?;
    repository::cleanup_expired_sessions::delete_batch_on_connection(connection, &cleanup).await
}
