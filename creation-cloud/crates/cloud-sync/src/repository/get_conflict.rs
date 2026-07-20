//! 按账号所有权读取单个未解决同步冲突。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::SyncConflict;

use super::{ConflictRow, conflict_from_row, storage};

pub(crate) async fn get_conflict(
    pool: &PgPool,
    account_id: Uuid,
    conflict_id: Uuid,
) -> AppResult<SyncConflict> {
    let row = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, client_mutation_id, base_revision, current_revision, attempted_changes, created_at \
         FROM sync_conflicts WHERE account_id = $1 AND id = $2 AND resolved_at IS NULL",
    )
    .bind(account_id)
    .bind(conflict_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法读取同步冲突"))?
    .ok_or_else(|| AppError::NotFound("同步冲突不存在".to_owned()))?;
    conflict_from_row(row)
}
