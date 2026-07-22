//! 取得独立任务锁后，将崩溃遗留的 running 记录收敛为 interrupted。

use cloud_domain::AppResult;
use sqlx::{Connection, PgConnection};

use crate::MaintenanceTask;

use super::{map_storage, trim};

pub(crate) async fn recover_interrupted(
    connection: &mut PgConnection,
    task: MaintenanceTask,
) -> AppResult<u64> {
    let mut transaction = connection.begin().await.map_err(map_storage)?;
    let history = trim::lock_task_history(&mut transaction, task).await?;
    let interrupted = sqlx::query(
        r#"
        UPDATE maintenance_task_runs
        SET outcome = 'interrupted', error_code = 'interrupted', finished_at = now()
        WHERE task_name = $1 AND outcome = 'running'
        "#,
    )
    .bind(task.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(map_storage)?
    .rows_affected();
    if interrupted > 0 {
        sqlx::query(
            r#"
            UPDATE maintenance_task_state
            SET active_run_id = NULL,
                consecutive_failures = consecutive_failures + 1,
                updated_at = now()
            WHERE task_name = $1
            "#,
        )
        .bind(task.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(map_storage)?;
    }
    trim::completed_history(&mut transaction, &history, None).await?;
    transaction.commit().await.map_err(map_storage)?;
    Ok(interrupted)
}
