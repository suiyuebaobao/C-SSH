//! 把固定任务分派到对应业务 Service 的公开维护用例。

mod backup_freshness;
mod download_aggregation;
mod expired_sessions;
mod published_asset_inspection;
mod sync_retention;

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_maintenance::{MaintenanceTask, TaskExecutionReport};
use sqlx::PgConnection;

use crate::{maintenance::progress::CommittedProgress, services::AppServices};

pub struct TaskContext {
    pub now: DateTime<Utc>,
    pub cutoff_at: Option<DateTime<Utc>>,
    pub active_cutoff_at: Option<DateTime<Utc>>,
}

pub async fn execute(
    task: MaintenanceTask,
    services: &AppServices,
    config: &cloud_config::MaintenanceConfig,
    context: &TaskContext,
    connection: &mut PgConnection,
    progress: &CommittedProgress,
) -> AppResult<TaskExecutionReport> {
    match task {
        MaintenanceTask::ExpiredSessions => {
            expired_sessions::execute(
                services,
                config,
                connection,
                progress,
                required_cutoff(context)?,
            )
            .await
        }
        MaintenanceTask::SyncRetention => {
            sync_retention::execute(
                services,
                config,
                connection,
                progress,
                required_cutoff(context)?,
                context.active_cutoff_at.ok_or_else(missing_cutoff)?,
            )
            .await
        }
        MaintenanceTask::DownloadAggregation => {
            download_aggregation::execute(services, config, connection, progress).await
        }
        MaintenanceTask::PublishedAssetInspection => {
            published_asset_inspection::execute(services, connection, progress).await
        }
        MaintenanceTask::BackupFreshness => {
            Ok(backup_freshness::execute(config, context.now).await)
        }
    }
}

fn required_cutoff(context: &TaskContext) -> AppResult<DateTime<Utc>> {
    context.cutoff_at.ok_or_else(missing_cutoff)
}

fn missing_cutoff() -> AppError {
    AppError::Internal("维护任务缺少已计算 cutoff".to_owned())
}
