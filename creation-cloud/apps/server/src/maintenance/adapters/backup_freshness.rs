//! 调用共享只读清单检查器并映射备份观察码与文件计数。

use chrono::{DateTime, Utc};
use cloud_maintenance::{ObservationCode, TaskExecutionReport, check_latest_backup};

pub async fn execute(
    config: &cloud_config::MaintenanceConfig,
    now: DateTime<Utc>,
) -> TaskExecutionReport {
    let checked =
        check_latest_backup(&config.backup_root, config.backup_freshness_window, now).await;
    let healthy =
        u64::from(checked.observation == ObservationCode::Healthy) * checked.checked_file_count;
    let issues = u64::from(checked.observation != ObservationCode::Healthy);
    TaskExecutionReport {
        examined_count: checked.checked_file_count,
        healthy_count: healthy,
        issue_count: issues,
        observation: Some(checked.observation),
        ..TaskExecutionReport::default()
    }
}
