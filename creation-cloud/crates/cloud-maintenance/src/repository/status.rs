//! 在单条 PostgreSQL 快照查询中组合固定任务状态与最近一次运行。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::{Executor, FromRow, PgConnection, Postgres};
use uuid::Uuid;

use crate::{MaintenanceStatus, MaintenanceTask, ObservationCode, RunRecord};

use super::{RunRow, count, map_storage, parse_optional};

const STATUS_SQL: &str = r#"
    SELECT
        state.active_run_id,
        state.last_success_at,
        state.consecutive_failures,
        state.last_observation_code,
        state.updated_at,
        latest.run_id AS latest_run_id,
        latest.task_name AS latest_task_name,
        latest.trigger_kind AS latest_trigger_kind,
        latest.instance_id AS latest_instance_id,
        latest.outcome AS latest_outcome,
        latest.observation_code AS latest_observation_code,
        latest.error_code AS latest_error_code,
        latest.cutoff_at AS latest_cutoff_at,
        latest.active_cutoff_at AS latest_active_cutoff_at,
        latest.started_at AS latest_started_at,
        latest.finished_at AS latest_finished_at,
        latest.examined_count AS latest_examined_count,
        latest.changed_count AS latest_changed_count,
        latest.healthy_count AS latest_healthy_count,
        latest.issue_count AS latest_issue_count
    FROM maintenance_task_state AS state
    LEFT JOIN LATERAL (
        SELECT
            run_id, task_name, trigger_kind, instance_id, outcome,
            observation_code, error_code, cutoff_at, active_cutoff_at,
            started_at, finished_at, examined_count, changed_count,
            healthy_count, issue_count
        FROM maintenance_task_runs
        WHERE task_name = state.task_name
        ORDER BY started_at DESC, run_id DESC
        LIMIT 1
    ) AS latest ON TRUE
    WHERE state.task_name = $1
"#;

#[derive(FromRow)]
struct StatusRow {
    active_run_id: Option<Uuid>,
    last_success_at: Option<DateTime<Utc>>,
    consecutive_failures: i64,
    last_observation_code: Option<String>,
    updated_at: DateTime<Utc>,
    latest_run_id: Option<Uuid>,
    latest_task_name: Option<String>,
    latest_trigger_kind: Option<String>,
    latest_instance_id: Option<Uuid>,
    latest_outcome: Option<String>,
    latest_observation_code: Option<String>,
    latest_error_code: Option<String>,
    latest_cutoff_at: Option<DateTime<Utc>>,
    latest_active_cutoff_at: Option<DateTime<Utc>>,
    latest_started_at: Option<DateTime<Utc>>,
    latest_finished_at: Option<DateTime<Utc>>,
    latest_examined_count: Option<i64>,
    latest_changed_count: Option<i64>,
    latest_healthy_count: Option<i64>,
    latest_issue_count: Option<i64>,
}

pub(crate) async fn status(pool: &PgPool, task: MaintenanceTask) -> AppResult<MaintenanceStatus> {
    fetch_status(pool, task).await
}

pub(crate) async fn status_on(
    connection: &mut PgConnection,
    task: MaintenanceTask,
) -> AppResult<MaintenanceStatus> {
    fetch_status(&mut *connection, task).await
}

async fn fetch_status<'executor, E>(
    executor: E,
    task: MaintenanceTask,
) -> AppResult<MaintenanceStatus>
where
    E: Executor<'executor, Database = Postgres>,
{
    sqlx::query_as::<_, StatusRow>(STATUS_SQL)
        .bind(task.as_str())
        .fetch_optional(executor)
        .await
        .map_err(map_storage)?
        .ok_or_else(|| AppError::Storage("固定维护任务状态缺失".to_owned()))?
        .into_status(task)
}

impl StatusRow {
    fn into_status(self, task: MaintenanceTask) -> AppResult<MaintenanceStatus> {
        let latest_attempt = self.latest_attempt()?;
        Ok(MaintenanceStatus {
            task,
            active_run_id: self.active_run_id,
            latest_attempt,
            last_success_at: self.last_success_at,
            consecutive_failures: count(self.consecutive_failures)?,
            last_observation: parse_optional(
                self.last_observation_code,
                ObservationCode::parse,
                "观察码",
            )?,
            updated_at: self.updated_at,
        })
    }

    fn latest_attempt(&self) -> AppResult<Option<RunRecord>> {
        let Some(run_id) = self.latest_run_id else {
            return Ok(None);
        };
        RunRow {
            run_id,
            task_name: required(self.latest_task_name.clone(), "任务名")?,
            trigger_kind: required(self.latest_trigger_kind.clone(), "触发类型")?,
            instance_id: required(self.latest_instance_id, "实例标识")?,
            outcome: required(self.latest_outcome.clone(), "运行结果")?,
            observation_code: self.latest_observation_code.clone(),
            error_code: self.latest_error_code.clone(),
            cutoff_at: self.latest_cutoff_at,
            active_cutoff_at: self.latest_active_cutoff_at,
            started_at: required(self.latest_started_at, "开始时间")?,
            finished_at: self.latest_finished_at,
            examined_count: required(self.latest_examined_count, "检查计数")?,
            changed_count: required(self.latest_changed_count, "变更计数")?,
            healthy_count: required(self.latest_healthy_count, "健康计数")?,
            issue_count: required(self.latest_issue_count, "问题计数")?,
        }
        .try_into()
        .map(Some)
    }
}

fn required<T>(value: Option<T>, field: &'static str) -> AppResult<T> {
    value.ok_or_else(|| AppError::Storage(format!("维护任务最近运行{field}状态缺失")))
}

#[cfg(test)]
mod status_tests;
