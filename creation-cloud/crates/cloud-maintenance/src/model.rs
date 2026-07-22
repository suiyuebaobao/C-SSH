//! 定义维护运行、状态、固定结果码和业务适配器的无敏感正文报告。

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::MaintenanceTask;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
    Startup,
    Scheduled,
    Manual,
}

impl RunTrigger {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "startup" => Some(Self::Startup),
            "scheduled" => Some(Self::Scheduled),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    Running,
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
    SkippedLocked,
    Interrupted,
}

impl RunOutcome {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::SkippedLocked => "skipped_locked",
            Self::Interrupted => "interrupted",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "timed_out" => Some(Self::TimedOut),
            "cancelled" => Some(Self::Cancelled),
            "skipped_locked" => Some(Self::SkippedLocked),
            "interrupted" => Some(Self::Interrupted),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationCode {
    Healthy,
    Missing,
    Stale,
    Invalid,
    IssuesDetected,
}

impl ObservationCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Missing => "missing",
            Self::Stale => "stale",
            Self::Invalid => "invalid",
            Self::IssuesDetected => "issues_detected",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "healthy" => Some(Self::Healthy),
            "missing" => Some(Self::Missing),
            "stale" => Some(Self::Stale),
            "invalid" => Some(Self::Invalid),
            "issues_detected" => Some(Self::IssuesDetected),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    TaskFailed,
    TimedOut,
    Cancelled,
    LockHeld,
    Interrupted,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TaskFailed => "task_failed",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::LockHeld => "lock_held",
            Self::Interrupted => "interrupted",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "task_failed" => Some(Self::TaskFailed),
            "timed_out" => Some(Self::TimedOut),
            "cancelled" => Some(Self::Cancelled),
            "lock_held" => Some(Self::LockHeld),
            "interrupted" => Some(Self::Interrupted),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunStart {
    pub run_id: Uuid,
    pub task: MaintenanceTask,
    pub trigger: RunTrigger,
    pub instance_id: Uuid,
    pub cutoff_at: Option<DateTime<Utc>>,
    pub active_cutoff_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct RunCompletion {
    pub run_id: Uuid,
    pub task: MaintenanceTask,
    pub outcome: RunOutcome,
    pub observation: Option<ObservationCode>,
    pub error: Option<ErrorCode>,
    pub report: TaskExecutionReport,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TaskExecutionReport {
    pub examined_count: u64,
    pub changed_count: u64,
    pub healthy_count: u64,
    pub issue_count: u64,
    pub observation: Option<ObservationCode>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunRecord {
    pub run_id: Uuid,
    pub task: MaintenanceTask,
    pub trigger: RunTrigger,
    pub instance_id: Uuid,
    pub outcome: RunOutcome,
    pub observation: Option<ObservationCode>,
    pub error: Option<ErrorCode>,
    pub cutoff_at: Option<DateTime<Utc>>,
    pub active_cutoff_at: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub examined_count: u64,
    pub changed_count: u64,
    pub healthy_count: u64,
    pub issue_count: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MaintenanceStatus {
    pub task: MaintenanceTask,
    pub active_run_id: Option<Uuid>,
    pub latest_attempt: Option<RunRecord>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub consecutive_failures: u64,
    pub last_observation: Option<ObservationCode>,
    pub updated_at: DateTime<Utc>,
}
