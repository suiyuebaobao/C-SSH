//! 验证维护状态单快照查询的完整、空历史与损坏行转换边界。

use super::*;
use crate::{RunOutcome, RunTrigger};

#[test]
fn joined_status_row_builds_active_latest_attempt_consistently() {
    let active_run_id = Uuid::from_u128(1);
    let instance_id = Uuid::from_u128(2);
    let updated_at = timestamp("2026-07-22T01:02:03Z");
    let status = status_row(Some(active_run_id), updated_at)
        .with_latest(active_run_id, instance_id, updated_at)
        .into_status(MaintenanceTask::ExpiredSessions)
        .expect("同一查询返回的完整状态应能转换");

    assert_eq!(status.active_run_id, Some(active_run_id));
    let latest = status.latest_attempt.expect("应包含最近运行");
    assert_eq!(latest.run_id, active_run_id);
    assert_eq!(latest.instance_id, instance_id);
    assert_eq!(latest.trigger, RunTrigger::Manual);
    assert_eq!(latest.outcome, RunOutcome::Running);
}

#[test]
fn joined_status_row_allows_task_without_run_history() {
    let status = status_row(None, timestamp("2026-07-22T02:03:04Z"))
        .into_status(MaintenanceTask::BackupFreshness)
        .expect("无运行历史的固定任务状态应有效");

    assert!(status.latest_attempt.is_none());
    assert_eq!(status.consecutive_failures, 0);
}

#[test]
fn joined_status_row_rejects_partial_latest_attempt() {
    let mut row = status_row(None, timestamp("2026-07-22T03:04:05Z"));
    row.latest_run_id = Some(Uuid::from_u128(3));

    assert!(row.into_status(MaintenanceTask::SyncRetention).is_err());
}

#[test]
fn status_sql_uses_one_deterministically_ordered_snapshot_query() {
    assert!(STATUS_SQL.contains("LEFT JOIN LATERAL"));
    assert!(STATUS_SQL.contains("ORDER BY started_at DESC, run_id DESC"));
}

fn status_row(active_run_id: Option<Uuid>, updated_at: DateTime<Utc>) -> StatusRow {
    StatusRow {
        active_run_id,
        last_success_at: None,
        consecutive_failures: 0,
        last_observation_code: None,
        updated_at,
        latest_run_id: None,
        latest_task_name: None,
        latest_trigger_kind: None,
        latest_instance_id: None,
        latest_outcome: None,
        latest_observation_code: None,
        latest_error_code: None,
        latest_cutoff_at: None,
        latest_active_cutoff_at: None,
        latest_started_at: None,
        latest_finished_at: None,
        latest_examined_count: None,
        latest_changed_count: None,
        latest_healthy_count: None,
        latest_issue_count: None,
    }
}

impl StatusRow {
    fn with_latest(mut self, run_id: Uuid, instance_id: Uuid, started_at: DateTime<Utc>) -> Self {
        self.latest_run_id = Some(run_id);
        self.latest_task_name = Some(MaintenanceTask::ExpiredSessions.as_str().to_owned());
        self.latest_trigger_kind = Some(RunTrigger::Manual.as_str().to_owned());
        self.latest_instance_id = Some(instance_id);
        self.latest_outcome = Some(RunOutcome::Running.as_str().to_owned());
        self.latest_started_at = Some(started_at);
        self.latest_examined_count = Some(0);
        self.latest_changed_count = Some(0);
        self.latest_healthy_count = Some(0);
        self.latest_issue_count = Some(0);
        self
    }
}

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("测试时间必须有效")
        .with_timezone(&Utc)
}
