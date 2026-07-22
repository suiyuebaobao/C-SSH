//! 汇总维护运行状态按动作拆分的 PostgreSQL 仓储。

mod complete;
mod read;
mod recover;
mod skipped;
mod start;
mod status;
mod trim;

pub(crate) use complete::complete;
pub(crate) use read::{database_now, database_now_on, read_run, read_run_on};
pub(crate) use recover::recover_interrupted;
pub(crate) use skipped::{record_skipped_locked, record_skipped_locked_on};
pub(crate) use start::start;
pub(crate) use status::{status, status_on};

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{ErrorCode, ObservationCode, RunOutcome, RunRecord, RunTrigger};

#[derive(FromRow)]
pub(crate) struct RunRow {
    run_id: Uuid,
    task_name: String,
    trigger_kind: String,
    instance_id: Uuid,
    outcome: String,
    observation_code: Option<String>,
    error_code: Option<String>,
    cutoff_at: Option<DateTime<Utc>>,
    active_cutoff_at: Option<DateTime<Utc>>,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    examined_count: i64,
    changed_count: i64,
    healthy_count: i64,
    issue_count: i64,
}

impl TryFrom<RunRow> for RunRecord {
    type Error = AppError;

    fn try_from(row: RunRow) -> AppResult<Self> {
        Ok(Self {
            run_id: row.run_id,
            task: row
                .task_name
                .parse()
                .map_err(|()| invalid_state("任务名"))?,
            trigger: RunTrigger::parse(&row.trigger_kind)
                .ok_or_else(|| invalid_state("触发类型"))?,
            instance_id: row.instance_id,
            outcome: RunOutcome::parse(&row.outcome).ok_or_else(|| invalid_state("运行结果"))?,
            observation: parse_optional(row.observation_code, ObservationCode::parse, "观察码")?,
            error: parse_optional(row.error_code, ErrorCode::parse, "错误码")?,
            cutoff_at: row.cutoff_at,
            active_cutoff_at: row.active_cutoff_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
            examined_count: count(row.examined_count)?,
            changed_count: count(row.changed_count)?,
            healthy_count: count(row.healthy_count)?,
            issue_count: count(row.issue_count)?,
        })
    }
}

pub(crate) fn map_storage(_: sqlx::Error) -> AppError {
    AppError::Storage("维护任务状态存储失败".to_owned())
}

pub(crate) fn count(value: i64) -> AppResult<u64> {
    u64::try_from(value).map_err(|_| invalid_state("计数"))
}

pub(crate) fn database_count(value: u64) -> AppResult<i64> {
    i64::try_from(value).map_err(|_| AppError::Validation("维护任务计数超出范围".to_owned()))
}

fn parse_optional<T>(
    value: Option<String>,
    parse: impl Fn(&str) -> Option<T>,
    name: &'static str,
) -> AppResult<Option<T>> {
    value
        .map(|value| parse(&value).ok_or_else(|| invalid_state(name)))
        .transpose()
}

fn invalid_state(name: &'static str) -> AppError {
    AppError::Storage(format!("维护任务{name}状态无效"))
}
