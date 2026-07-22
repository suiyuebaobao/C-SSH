//! 在同步 actor 的数据库事务内记录设备活动与已确认修订。
//! 只有增量 pull 可以提升确认游标，其它动作只能刷新活动时间。

use cloud_domain::AppResult;
use cloud_store::{Postgres, Transaction};

use crate::actor::SyncActor;

use super::storage;

pub(crate) const TOUCH_CHECKPOINT_SQL: &str = r#"
INSERT INTO sync_device_checkpoints
    (account_id, device_id, acknowledged_revision, last_sync_at, updated_at)
VALUES ($1, $2, 0, now(), now())
ON CONFLICT (account_id, device_id) DO UPDATE SET
    last_sync_at = EXCLUDED.last_sync_at,
    updated_at = EXCLUDED.updated_at
"#;

pub(crate) const ACKNOWLEDGE_CHECKPOINT_SQL: &str = r#"
INSERT INTO sync_device_checkpoints
    (account_id, device_id, acknowledged_revision, last_sync_at, updated_at)
VALUES ($1, $2, $3, now(), now())
ON CONFLICT (account_id, device_id) DO UPDATE SET
    acknowledged_revision = GREATEST(
        sync_device_checkpoints.acknowledged_revision,
        EXCLUDED.acknowledged_revision
    ),
    last_sync_at = EXCLUDED.last_sync_at,
    updated_at = EXCLUDED.updated_at
"#;

pub(crate) async fn touch(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
) -> AppResult<()> {
    sqlx::query(TOUCH_CHECKPOINT_SQL)
        .bind(actor.account_id())
        .bind(actor.device_id())
        .execute(&mut **transaction)
        .await
        .map_err(storage("无法更新同步设备活动"))?;
    Ok(())
}

pub(crate) async fn acknowledge_incremental(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    since_revision: i64,
) -> AppResult<()> {
    sqlx::query(ACKNOWLEDGE_CHECKPOINT_SQL)
        .bind(actor.account_id())
        .bind(actor.device_id())
        .bind(since_revision)
        .execute(&mut **transaction)
        .await
        .map_err(storage("无法更新同步设备确认游标"))?;
    Ok(())
}
