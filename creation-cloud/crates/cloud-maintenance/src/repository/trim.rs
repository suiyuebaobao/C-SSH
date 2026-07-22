//! 将每项维护任务的已完成运行历史串行收敛到最近一百条。

use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::MaintenanceTask;

use super::map_storage;

const COMPLETED_HISTORY_LIMIT: i64 = 100;
const LOCK_TASK_STATE_SQL: &str = r#"
    SELECT active_run_id
    FROM maintenance_task_state
    WHERE task_name = $1
    FOR UPDATE
"#;
const DELETE_OLD_COMPLETED_HISTORY_SQL: &str = r#"
    DELETE FROM maintenance_task_runs
    WHERE run_id IN (
        SELECT run_id
        FROM maintenance_task_runs
        WHERE task_name = $1 AND outcome <> 'running'
        ORDER BY
            CASE WHEN run_id = $2 OR run_id = $3 THEN 0 ELSE 1 END,
            started_at DESC,
            run_id DESC
        OFFSET $4
    )
"#;

pub(crate) struct LockedTaskHistory {
    task: MaintenanceTask,
    active_run_id: Option<Uuid>,
}

pub(crate) async fn lock_task_history(
    transaction: &mut Transaction<'_, Postgres>,
    task: MaintenanceTask,
) -> AppResult<LockedTaskHistory> {
    // 必须先独立取得行锁，后续历史写和 DELETE 才按任务串行，并在等待后使用各自的新语句快照。
    let active_run_id = sqlx::query_scalar::<_, Option<Uuid>>(LOCK_TASK_STATE_SQL)
        .bind(task.as_str())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(map_storage)?
        .ok_or_else(|| AppError::Storage("固定维护任务状态缺失".to_owned()))?;
    Ok(LockedTaskHistory {
        task,
        active_run_id,
    })
}

pub(crate) async fn completed_history(
    transaction: &mut Transaction<'_, Postgres>,
    locked: &LockedTaskHistory,
    preserve_run_id: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(DELETE_OLD_COMPLETED_HISTORY_SQL)
        .bind(locked.task.as_str())
        .bind(preserve_run_id)
        .bind(locked.active_run_id)
        .bind(COMPLETED_HISTORY_LIMIT)
        .execute(&mut **transaction)
        .await
        .map_err(map_storage)?;
    Ok(())
}

#[cfg(test)]
mod trim_tests;
