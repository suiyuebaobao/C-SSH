//! 调用同步域保留用例并汇总五类真实删除计数。

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
    retention_cutoff: DateTime<Utc>,
    active_cutoff: DateTime<Utc>,
) -> AppResult<TaskExecutionReport> {
    loop {
        let report = services
            .sync
            .run_retention_batch_on_connection(
                &mut *connection,
                retention_cutoff,
                active_cutoff,
                config.cleanup_batch_size,
            )
            .await?;
        let changed = report
            .tombstones_deleted
            .saturating_add(report.record_versions_deleted)
            .saturating_add(report.applied_mutations_deleted)
            .saturating_add(report.resolved_conflicts_deleted)
            .saturating_add(report.conflict_mutations_deleted);
        progress.add(TaskExecutionReport {
            examined_count: changed,
            changed_count: changed,
            ..TaskExecutionReport::default()
        });
        if changed == 0 {
            return Ok(progress.snapshot());
        }
    }
}
