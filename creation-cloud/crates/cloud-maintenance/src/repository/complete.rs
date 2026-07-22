//! 以 run_id CAS 原子完成运行、更新当前状态并裁剪历史。

use cloud_domain::{AppError, AppResult};
use sqlx::{Connection, PgConnection};

use crate::{RunCompletion, RunOutcome, RunRecord, connection_bounds::LOCK_WAIT_TIMEOUT_SECONDS};

use super::{database_count, map_storage, read::read_run_in_transaction, trim};

pub(crate) async fn complete(
    connection: &mut PgConnection,
    completion: &RunCompletion,
) -> AppResult<RunRecord> {
    if matches!(
        completion.outcome,
        RunOutcome::Running | RunOutcome::SkippedLocked | RunOutcome::Interrupted
    ) {
        return Err(AppError::Validation("运行完成结果无效".to_owned()));
    }
    let mut transaction = connection.begin().await.map_err(map_storage)?;
    // 终态写必须在统一退出窗口内结束，不能无限等待外部行锁或异常慢语句。
    let lock_timeout = format!("{LOCK_WAIT_TIMEOUT_SECONDS}s");
    sqlx::query_scalar::<_, String>("SELECT set_config('lock_timeout', $1, true)")
        .bind(lock_timeout)
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_storage)?;
    sqlx::query("SET LOCAL statement_timeout = '10s'")
        .execute(&mut *transaction)
        .await
        .map_err(map_storage)?;
    let history = trim::lock_task_history(&mut transaction, completion.task).await?;
    let updated = sqlx::query(
        r#"
        UPDATE maintenance_task_runs
        SET outcome = $3, observation_code = $4, error_code = $5,
            finished_at = now(), examined_count = $6, changed_count = $7,
            healthy_count = $8, issue_count = $9
        WHERE task_name = $1 AND run_id = $2 AND outcome = 'running'
        "#,
    )
    .bind(completion.task.as_str())
    .bind(completion.run_id)
    .bind(completion.outcome.as_str())
    .bind(completion.observation.map(|value| value.as_str()))
    .bind(completion.error.map(|value| value.as_str()))
    .bind(database_count(completion.report.examined_count)?)
    .bind(database_count(completion.report.changed_count)?)
    .bind(database_count(completion.report.healthy_count)?)
    .bind(database_count(completion.report.issue_count)?)
    .execute(&mut *transaction)
    .await
    .map_err(map_storage)?;
    if updated.rows_affected() != 1 {
        let record = read_run_in_transaction(&mut transaction, completion.run_id).await?;
        if record.task != completion.task || record.outcome == RunOutcome::Running {
            return Err(AppError::Conflict("维护运行已经结束或不存在".to_owned()));
        }
        transaction.commit().await.map_err(map_storage)?;
        return Ok(record);
    }
    let state = sqlx::query(
        r#"
        UPDATE maintenance_task_state
        SET active_run_id = NULL,
            last_success_at = CASE WHEN $3 = 'succeeded' THEN now() ELSE last_success_at END,
            consecutive_failures = CASE
                WHEN $3 = 'succeeded' THEN 0
                WHEN $3 IN ('failed', 'timed_out') THEN consecutive_failures + 1
                ELSE consecutive_failures
            END,
            last_observation_code = COALESCE($4, last_observation_code),
            updated_at = now()
        WHERE task_name = $1 AND active_run_id = $2
        "#,
    )
    .bind(completion.task.as_str())
    .bind(completion.run_id)
    .bind(completion.outcome.as_str())
    .bind(completion.observation.map(|value| value.as_str()))
    .execute(&mut *transaction)
    .await
    .map_err(map_storage)?;
    if state.rows_affected() != 1 {
        return Err(AppError::Conflict(
            "维护任务状态已被其它运行接管".to_owned(),
        ));
    }
    trim::completed_history(&mut transaction, &history, Some(completion.run_id)).await?;
    let record = read_run_in_transaction(&mut transaction, completion.run_id).await?;
    transaction.commit().await.map_err(map_storage)?;
    Ok(record)
}
