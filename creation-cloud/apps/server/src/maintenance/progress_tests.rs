//! 验证维护任务已提交进度的累计结果与观察码隔离边界。

use super::*;

#[test]
fn committed_progress_accumulates_without_copying_observation_codes() {
    let progress = CommittedProgress::default();
    progress.add(TaskExecutionReport {
        examined_count: 3,
        changed_count: 2,
        healthy_count: 1,
        issue_count: 1,
        observation: Some(cloud_maintenance::ObservationCode::IssuesDetected),
    });
    progress.add(TaskExecutionReport {
        examined_count: 4,
        changed_count: 1,
        ..TaskExecutionReport::default()
    });
    assert_eq!(
        progress.snapshot(),
        TaskExecutionReport {
            examined_count: 7,
            changed_count: 3,
            healthy_count: 1,
            issue_count: 1,
            observation: None,
        }
    );
}
