//! 在账号与会话锁内创建或重绑 active 设备，并返回数据库权威值。

use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::model::{CreateDeviceOutcome, Device, DeviceRow};

use super::error;

pub(crate) const LOCK_ACCOUNT_SQL: &str =
    "SELECT id FROM accounts WHERE id = $1 AND status = 'active' FOR UPDATE";
pub(crate) const LOCK_SESSION_SQL: &str = "SELECT device_id FROM sessions \
     WHERE id = $1 AND account_id = $2 AND expires_at > now() FOR UPDATE";

pub(crate) async fn bind(
    pool: &PgPool,
    session: &AuthenticatedSession,
    name: &str,
    platform: &str,
    public_id: &str,
) -> AppResult<CreateDeviceOutcome> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let account_exists = sqlx::query_scalar::<_, Uuid>(LOCK_ACCOUNT_SQL)
        .bind(session.account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .is_some();
    if !account_exists {
        return Err(AppError::Unauthorized("账号不可用".to_owned()));
    }

    let bound_device = sqlx::query_scalar::<_, Option<Uuid>>(LOCK_SESSION_SQL)
        .bind(session.session_id)
        .bind(session.account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::storage)?
        .ok_or_else(|| AppError::Unauthorized("会话无效或已过期".to_owned()))?;

    let existing = sqlx::query_as::<_, DeviceRow>(
        "SELECT id, account_id, name, platform, public_id, last_seen_at, revoked_at, \
                created_at, updated_at FROM devices \
         WHERE account_id = $1 AND public_id = $2 FOR UPDATE",
    )
    .bind(session.account_id)
    .bind(public_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;

    let (device, created) = match existing {
        Some(row) => {
            if row.6.is_some() {
                return Err(AppError::Conflict(
                    "已撤销的设备标识不能重新登记".to_owned(),
                ));
            }
            if row.3 != platform {
                return Err(AppError::Conflict("设备平台与既有登记不一致".to_owned()));
            }
            if bound_device.is_some_and(|device_id| device_id != row.0) {
                return Err(AppError::Conflict("当前会话已绑定其它设备".to_owned()));
            }
            let row = sqlx::query_as::<_, DeviceRow>(
                "UPDATE devices SET name = $3, last_seen_at = now(), updated_at = now() \
                 WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL \
                 RETURNING id, account_id, name, platform, public_id, last_seen_at, \
                           revoked_at, created_at, updated_at",
            )
            .bind(session.account_id)
            .bind(row.0)
            .bind(name)
            .fetch_one(&mut *transaction)
            .await
            .map_err(error::storage)?;
            (Device::from_row(row), false)
        }
        None => {
            if bound_device.is_some() {
                return Err(AppError::Conflict("当前会话已绑定其它设备".to_owned()));
            }
            let row = sqlx::query_as::<_, DeviceRow>(
                "INSERT INTO devices \
                 (id, account_id, name, platform, public_id, last_seen_at) \
                 VALUES ($1, $2, $3, $4, $5, now()) \
                 RETURNING id, account_id, name, platform, public_id, last_seen_at, \
                           revoked_at, created_at, updated_at",
            )
            .bind(Uuid::now_v7())
            .bind(session.account_id)
            .bind(name)
            .bind(platform)
            .bind(public_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(error::create)?;
            (Device::from_row(row), true)
        }
    };

    let updated = sqlx::query(
        "UPDATE sessions SET device_id = $3 \
         WHERE id = $1 AND account_id = $2 AND expires_at > now()",
    )
    .bind(session.session_id)
    .bind(session.account_id)
    .bind(device.id)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if updated.rows_affected() != 1 {
        return Err(AppError::Unauthorized("会话无效或已过期".to_owned()));
    }
    transaction.commit().await.map_err(error::storage)?;
    Ok(CreateDeviceOutcome { device, created })
}
