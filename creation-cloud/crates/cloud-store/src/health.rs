//! 用最小查询确认数据库连接与查询通道可用。

use cloud_domain::{AppError, AppResult};
use sqlx::PgPool;

pub async fn health(pool: &PgPool) -> AppResult<()> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
        .map_err(|error| AppError::Storage(format!("PostgreSQL 健康检查失败: {error}")))
}
