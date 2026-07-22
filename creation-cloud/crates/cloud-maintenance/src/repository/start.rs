//! 原子插入运行记录并把固定任务状态绑定到新的活动 run_id。

use cloud_domain::{AppError, AppResult};
use sqlx::{Connection, PgConnection};

use crate::RunStart;

use super::map_storage;

pub(crate) async fn start(connection: &mut PgConnection, run: &RunStart) -> AppResult<()> {
    let mut transaction = connection.begin().await.map_err(map_storage)?;
    sqlx::query(
        r#"
        INSERT INTO maintenance_task_runs (
            run_id, task_name, trigger_kind, instance_id, outcome,
            cutoff_at, active_cutoff_at
        ) VALUES ($1, $2, $3, $4, 'running', $5, $6)
        "#,
    )
    .bind(run.run_id)
    .bind(run.task.as_str())
    .bind(run.trigger.as_str())
    .bind(run.instance_id)
    .bind(run.cutoff_at)
    .bind(run.active_cutoff_at)
    .execute(&mut *transaction)
    .await
    .map_err(map_storage)?;
    let updated = sqlx::query(
        r#"
        UPDATE maintenance_task_state
        SET active_run_id = $2, updated_at = now()
        WHERE task_name = $1 AND active_run_id IS NULL
        "#,
    )
    .bind(run.task.as_str())
    .bind(run.run_id)
    .execute(&mut *transaction)
    .await
    .map_err(map_storage)?;
    if updated.rows_affected() != 1 {
        return Err(AppError::Conflict("维护任务已经存在活动运行".to_owned()));
    }
    transaction.commit().await.map_err(map_storage)
}
