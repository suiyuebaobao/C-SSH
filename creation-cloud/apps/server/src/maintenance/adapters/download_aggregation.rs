//! 调用下载域精确一次聚合并映射事件与 UTC 日桶计数。

use cloud_domain::AppResult;
use cloud_maintenance::TaskExecutionReport;
use sqlx::PgConnection;

use crate::{maintenance::progress::CommittedProgress, services::AppServices};

pub async fn execute(
    services: &AppServices,
    config: &cloud_config::MaintenanceConfig,
    connection: &mut PgConnection,
    progress: &CommittedProgress,
) -> AppResult<TaskExecutionReport> {
    loop {
        let report = services
            .download
            .aggregate_download_events_with_connection(&mut *connection, config.cleanup_batch_size)
            .await?;
        progress.add(TaskExecutionReport {
            examined_count: report.processed_events,
            changed_count: report.updated_buckets,
            ..TaskExecutionReport::default()
        });
        if report.processed_events < u64::from(config.cleanup_batch_size) {
            break;
        }
    }
    Ok(progress.snapshot())
}
