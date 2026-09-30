//! 验证取消会重分类未提交终态，并始终采用最早绝对截止时间。

use cloud_maintenance::TaskExecutionReport;
use uuid::Uuid;

use super::*;

fn completion() -> RunCompletion {
    RunCompletion {
        run_id: Uuid::now_v7(),
        task: MaintenanceTask::ExpiredSessions,
        outcome: RunOutcome::Succeeded,
        observation: Some(cloud_maintenance::ObservationCode::Healthy),
        error: None,
        report: TaskExecutionReport {
            changed_count: 7,
            ..TaskExecutionReport::default()
        },
    }
}

#[test]
fn cancellation_reclassifies_only_uncommitted_desired_terminal() {
    let mut completion = completion();
    mark_cancelled(&mut completion);
    assert_eq!(completion.outcome, RunOutcome::Cancelled);
    assert_eq!(completion.error, Some(ErrorCode::Cancelled));
    assert_eq!(completion.observation, None);
    assert_eq!(completion.report.changed_count, 7);
}

#[test]
fn acknowledged_shutdown_still_uses_the_earliest_current_deadline() {
    let local = Instant::now() + Duration::from_secs(30);
    let requested = local - Duration::from_secs(5);
    let (_sender, shutdown) = watch::channel(ShutdownSignal::Requested(requested));
    assert_eq!(effective_deadline(&shutdown, local), requested);
}
