//! 创建边界清晰且连接数受控的 PostgreSQL 连接池。

use std::time::Duration;

use cloud_domain::{AppError, AppResult};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect(database_url: &str) -> AppResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(12)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect(database_url)
        .await
        .map_err(|error| AppError::Storage(format!("连接 PostgreSQL 失败: {error}")))
}
