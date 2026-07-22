//! 校验保留任务边界，并逐批执行有界事务直到当前候选清空。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use sqlx::PgConnection;

use crate::{RetentionReport, Service, model::retention::RetentionRequest, repository};

impl Service {
    /// 逐批清理达到保留期限且满足安全条件的同步数据，并返回累计分类计数。
    pub async fn run_retention(
        &self,
        retention_cutoff: DateTime<Utc>,
        active_cutoff: DateTime<Utc>,
        batch_size: u32,
    ) -> AppResult<RetentionReport> {
        let request = RetentionRequest::new(retention_cutoff, active_cutoff, batch_size)?;
        let mut total = RetentionReport::default();
        loop {
            let batch = repository::run_retention_batch(&self.pool, &request).await?;
            if batch.is_empty() {
                return Ok(total);
            }
            total.absorb(batch);
        }
    }

    /// 在调用方持有的 PostgreSQL 会话上执行单批保留事务。
    pub async fn run_retention_batch_on_connection(
        &self,
        connection: &mut PgConnection,
        retention_cutoff: DateTime<Utc>,
        active_cutoff: DateTime<Utc>,
        batch_size: u32,
    ) -> AppResult<RetentionReport> {
        let request = RetentionRequest::new(retention_cutoff, active_cutoff, batch_size)?;
        repository::run_retention_batch_on_connection(connection, &request).await
    }
}
