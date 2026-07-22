//! 按 run_id 读取单次维护运行，并提供统一数据库当前时间。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::RunRecord;

use super::{RunRow, map_storage};

const RUN_COLUMNS: &str = r#"
    run_id, task_name, trigger_kind, instance_id, outcome,
    observation_code, error_code, cutoff_at, active_cutoff_at,
    started_at, finished_at, examined_count, changed_count,
    healthy_count, issue_count
"#;

pub(crate) async fn database_now(pool: &PgPool) -> AppResult<DateTime<Utc>> {
    sqlx::query_scalar("SELECT now()")
        .fetch_one(pool)
        .await
        .map_err(map_storage)
}

pub(crate) async fn database_now_on(connection: &mut PgConnection) -> AppResult<DateTime<Utc>> {
    sqlx::query_scalar("SELECT now()")
        .fetch_one(connection)
        .await
        .map_err(map_storage)
}

pub(crate) async fn read_run(pool: &PgPool, run_id: Uuid) -> AppResult<RunRecord> {
    let query = format!("SELECT {RUN_COLUMNS} FROM maintenance_task_runs WHERE run_id = $1");
    sqlx::query_as::<_, RunRow>(&query)
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(map_storage)?
        .ok_or_else(|| AppError::NotFound("维护运行不存在".to_owned()))?
        .try_into()
}

pub(crate) async fn read_run_on(
    connection: &mut PgConnection,
    run_id: Uuid,
) -> AppResult<RunRecord> {
    let query = format!("SELECT {RUN_COLUMNS} FROM maintenance_task_runs WHERE run_id = $1");
    sqlx::query_as::<_, RunRow>(&query)
        .bind(run_id)
        .fetch_optional(connection)
        .await
        .map_err(map_storage)?
        .ok_or_else(|| AppError::NotFound("维护运行不存在".to_owned()))?
        .try_into()
}

pub(crate) async fn read_run_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    run_id: Uuid,
) -> AppResult<RunRecord> {
    let query = format!("SELECT {RUN_COLUMNS} FROM maintenance_task_runs WHERE run_id = $1");
    sqlx::query_as::<_, RunRow>(&query)
        .bind(run_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(map_storage)?
        .try_into()
}
