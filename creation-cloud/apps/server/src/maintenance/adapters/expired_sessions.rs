//! 调用认证域过期会话批处理并映射真实删除计数。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_maintenance::TaskExecutionReport;
use sqlx::PgConnection;

use crate::{maintenance::progress::CommittedProgress, services::AppServices};

pub async fn execute(
    services: &AppServices,
    config: &cloud_config::MaintenanceConfig,
    connection: &mut PgConnection,
    progress: &CommittedProgress,
    delete_before: DateTime<Utc>,
) -> AppResult<TaskExecutionReport> {
    loop {
        let current = services
            .auth
            .cleanup_expired_sessions_on_connection(
                &mut *connection,
                delete_before,
                config.cleanup_batch_size,
            )
            .await?;
        progress.add(TaskExecutionReport {
            examined_count: current,
            changed_count: current,
            ..TaskExecutionReport::default()
        });
        if current < u64::from(config.cleanup_batch_size) {
            break;
        }
    }
    Ok(progress.snapshot())
}
