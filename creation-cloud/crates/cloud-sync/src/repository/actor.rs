//! 在数据库边界重新确认同步 actor 的会话、账号与设备仍然有效。
//! 写事务锁定设备行，使同步提交与设备撤销在 PostgreSQL 中串行。

use cloud_domain::{AppError, AppResult};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::actor::SyncActor;

use super::storage;

pub(crate) const LOCK_ACTIVE_ACCOUNT_SQL: &str =
    "SELECT id FROM accounts WHERE id = $1 AND status = 'active' FOR UPDATE";
pub(crate) const LOCK_ACTIVE_DEVICE_SQL: &str = "SELECT id FROM devices \
    WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL FOR UPDATE";
pub(crate) const LOCK_ACTIVE_SESSION_SQL: &str = "SELECT id FROM sessions \
    WHERE id = $1 AND account_id = $2 AND device_id = $3 AND expires_at > now() FOR UPDATE";
pub(crate) const SHARE_ACTIVE_ACCOUNT_SQL: &str =
    "SELECT id FROM accounts WHERE id = $1 AND status = 'active' FOR SHARE";
pub(crate) const SHARE_ACTIVE_DEVICE_SQL: &str = "SELECT id FROM devices \
    WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL FOR SHARE";
pub(crate) const SHARE_ACTIVE_SESSION_SQL: &str = "SELECT id FROM sessions \
    WHERE id = $1 AND account_id = $2 AND device_id = $3 AND expires_at > now() FOR SHARE";

pub(crate) async fn lock_active(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
) -> AppResult<()> {
    // 与设备注册和撤销统一 account→device→session 锁序，避免交叉等待。
    let account = sqlx::query_scalar::<_, Uuid>(LOCK_ACTIVE_ACCOUNT_SQL)
        .bind(actor.account_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法锁定同步账号身份"))?;
    require_active(account)?;

    let device = sqlx::query_scalar::<_, Uuid>(LOCK_ACTIVE_DEVICE_SQL)
        .bind(actor.account_id())
        .bind(actor.device_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法锁定同步设备身份"))?;
    require_active(device)?;

    let session = sqlx::query_scalar::<_, Uuid>(LOCK_ACTIVE_SESSION_SQL)
        .bind(actor.session_id())
        .bind(actor.account_id())
        .bind(actor.device_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法锁定同步会话身份"))?;
    require_active(session)
}

pub(crate) async fn share_active(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
) -> AppResult<()> {
    // 共享锁允许同账号并发读取，同时阻止撤销或同步写在授权检查后穿越结果查询。
    let account = sqlx::query_scalar::<_, Uuid>(SHARE_ACTIVE_ACCOUNT_SQL)
        .bind(actor.account_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法确认同步账号身份"))?;
    require_active(account)?;

    let device = sqlx::query_scalar::<_, Uuid>(SHARE_ACTIVE_DEVICE_SQL)
        .bind(actor.account_id())
        .bind(actor.device_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法确认同步设备身份"))?;
    require_active(device)?;

    let session = sqlx::query_scalar::<_, Uuid>(SHARE_ACTIVE_SESSION_SQL)
        .bind(actor.session_id())
        .bind(actor.account_id())
        .bind(actor.device_id())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage("无法确认同步会话身份"))?;
    require_active(session)
}

fn require_active(active: Option<Uuid>) -> AppResult<()> {
    active
        .map(|_| ())
        .ok_or_else(|| AppError::Unauthorized("同步会话或设备已失效".to_owned()))
}
