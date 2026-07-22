//! 在未取得任务锁时只记录 skipped_locked 历史，不覆盖正在执行者的活动状态。

use cloud_domain::AppResult;
use cloud_store::{PgPool, Postgres, Transaction};
use sqlx::{Connection, PgConnection};

use crate::{ErrorCode, RunOutcome, RunRecord, RunStart};

use super::{map_storage, read::read_run_in_transaction, trim};

pub(crate) async fn record_skipped_locked(pool: &PgPool, run: &RunStart) -> AppResult<RunRecord> {
    let mut transaction = pool.begin().await.map_err(map_storage)?;
    let record = record_in_transaction(&mut transaction, run).await?;
    transaction.commit().await.map_err(map_storage)?;
    Ok(record)
}

pub(crate) async fn record_skipped_locked_on(
    connection: &mut PgConnection,
    run: &RunStart,
) -> AppResult<RunRecord> {
    let mut transaction = connection.begin().await.map_err(map_storage)?;
    let record = record_in_transaction(&mut transaction, run).await?;
    transaction.commit().await.map_err(map_storage)?;
    Ok(record)
}

async fn record_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    run: &RunStart,
) -> AppResult<RunRecord> {
    let history = trim::lock_task_history(transaction, run.task).await?;
    sqlx::query(
        r#"
        INSERT INTO maintenance_task_runs (
            run_id, task_name, trigger_kind, instance_id, outcome,
            error_code, cutoff_at, active_cutoff_at, finished_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
        "#,
    )
    .bind(run.run_id)
    .bind(run.task.as_str())
    .bind(run.trigger.as_str())
    .bind(run.instance_id)
    .bind(RunOutcome::SkippedLocked.as_str())
    .bind(ErrorCode::LockHeld.as_str())
    .bind(run.cutoff_at)
    .bind(run.active_cutoff_at)
    .execute(&mut **transaction)
    .await
    .map_err(map_storage)?;
    trim::completed_history(transaction, &history, Some(run.run_id)).await?;
    read_run_in_transaction(transaction, run.run_id).await
}
