//! 分页读取当前账号未解决的同步冲突。

use crate::{SyncConflict, actor::SyncActor};
use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;

use super::{ConflictRow, actor, checkpoint, conflict_from_row, storage};

pub(crate) async fn list_conflicts(
    pool: &PgPool,
    sync_actor: &SyncActor,
    page: PageQuery,
) -> AppResult<Page<SyncConflict>> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始同步冲突列表事务"))?;
    actor::share_active(&mut transaction, sync_actor).await?;
    checkpoint::touch(&mut transaction, sync_actor).await?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sync_conflicts \
         WHERE account_id = $1 AND resolved_at IS NULL",
    )
    .bind(sync_actor.account_id())
    .fetch_one(&mut *transaction)
    .await
    .map_err(storage("无法统计同步冲突"))?;
    let rows = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, client_mutation_id, base_revision, current_revision, attempted_changes, \
                source_device_id, created_at \
         FROM sync_conflicts WHERE account_id = $1 AND resolved_at IS NULL \
         ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3",
    )
    .bind(sync_actor.account_id())
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(&mut *transaction)
    .await
    .map_err(storage("无法读取同步冲突列表"))?;
    let items = rows
        .into_iter()
        .map(conflict_from_row)
        .collect::<AppResult<Vec<_>>>()?;
    transaction
        .commit()
        .await
        .map_err(storage("无法结束同步冲突列表事务"))?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}
