//! 验证运行器对被中断连接和非成功终态报告的纯状态转换。

use super::*;

#[test]
fn cancelled_and_timed_out_execution_never_reuses_interrupted_connection() {
    let deadline = Instant::now() + Duration::from_secs(20);
    for outcome in [RunOutcome::Cancelled, RunOutcome::TimedOut] {
        let result = terminal(
            outcome,
            Some(ErrorCode::Cancelled),
            TaskExecutionReport::default(),
            ConnectionDisposition::DiscardAndReacquire,
            deadline,
            outcome == RunOutcome::Cancelled,
        );
        assert_eq!(
            result.connection,
            ConnectionDisposition::DiscardAndReacquire
        );
        assert_eq!(result.terminal_deadline, deadline);
    }
}

#[test]
fn non_success_terminal_discards_observation_but_keeps_counts() {
    let deadline = Instant::now() + Duration::from_secs(20);
    let result = terminal(
        RunOutcome::Failed,
        Some(ErrorCode::TaskFailed),
        TaskExecutionReport {
            changed_count: 7,
            observation: Some(cloud_maintenance::ObservationCode::Healthy),
            ..TaskExecutionReport::default()
        },
        ConnectionDisposition::Reuse,
        deadline,
        false,
    );
    assert_eq!(result.report.changed_count, 7);
    assert_eq!(result.report.observation, None);
}
