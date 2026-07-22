//! 在已锁定账号修订的事务内统一应用同步变更并推进 revision。
//! push 与冲突解决必须复用这里，避免两套记录、墓碑和来源设备写入逻辑漂移。

use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::{SyncChange, SyncOperation, actor::SyncActor};

use super::storage;

pub(crate) async fn apply(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    current_revision: i64,
    changes: &[SyncChange],
) -> AppResult<i64> {
    let mut revision = current_revision;
    for change in changes {
        revision = revision
            .checked_add(1)
            .ok_or_else(|| AppError::Internal("同步修订号已超出范围".to_owned()))?;
        match change.operation {
            SyncOperation::Upsert => {
                upsert(transaction, actor, revision, change).await?;
            }
            SyncOperation::Delete => {
                delete(transaction, actor, revision, change).await?;
            }
        }
    }
    sqlx::query("UPDATE sync_states SET current_revision = $2 WHERE account_id = $1")
        .bind(actor.account_id())
        .bind(revision)
        .execute(&mut **transaction)
        .await
        .map_err(storage("无法推进同步修订号"))?;
    Ok(revision)
}

async fn upsert(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    revision: i64,
    change: &SyncChange,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sync_records \
         (id, account_id, namespace, record_key, revision, value, is_deleted, source_device_id) \
         VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7) \
         ON CONFLICT (account_id, namespace, record_key) DO UPDATE SET \
         revision = EXCLUDED.revision, value = EXCLUDED.value, \
         is_deleted = FALSE, source_device_id = EXCLUDED.source_device_id, updated_at = now()",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(&change.namespace)
    .bind(&change.key)
    .bind(revision)
    .bind(change.value.as_ref())
    .bind(actor.device_id())
    .execute(&mut **transaction)
    .await
    .map_err(storage("无法写入同步记录"))?;
    Ok(())
}

async fn delete(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    revision: i64,
    change: &SyncChange,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sync_records \
         (id, account_id, namespace, record_key, revision, value, is_deleted, source_device_id) \
         VALUES ($1, $2, $3, $4, $5, NULL, TRUE, $6) \
         ON CONFLICT (account_id, namespace, record_key) DO UPDATE SET \
         revision = EXCLUDED.revision, value = NULL, is_deleted = TRUE, \
         source_device_id = EXCLUDED.source_device_id, updated_at = now()",
    )
    .bind(Uuid::now_v7())
    .bind(actor.account_id())
    .bind(&change.namespace)
    .bind(&change.key)
    .bind(revision)
    .bind(actor.device_id())
    .execute(&mut **transaction)
    .await
    .map_err(storage("无法写入同步墓碑"))?;
    Ok(())
}
