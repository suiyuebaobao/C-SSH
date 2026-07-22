//! 按账号所有权读取单个未解决同步冲突。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{SyncConflict, actor::SyncActor};

use super::{ConflictRow, actor, checkpoint, conflict_from_row, storage};

pub(crate) async fn get_conflict(
    pool: &PgPool,
    sync_actor: &SyncActor,
    conflict_id: Uuid,
) -> AppResult<SyncConflict> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始同步冲突读取事务"))?;
    actor::share_active(&mut transaction, sync_actor).await?;
    checkpoint::touch(&mut transaction, sync_actor).await?;
    let row = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, client_mutation_id, base_revision, current_revision, attempted_changes, \
                source_device_id, created_at \
         FROM sync_conflicts WHERE account_id = $1 AND id = $2 AND resolved_at IS NULL",
    )
    .bind(sync_actor.account_id())
    .bind(conflict_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage("无法读取同步冲突"))?
    .ok_or_else(|| AppError::NotFound("同步冲突不存在".to_owned()))?;
    let conflict = conflict_from_row(row)?;
    transaction
        .commit()
        .await
        .map_err(storage("无法结束同步冲突读取事务"))?;
    Ok(conflict)
}
