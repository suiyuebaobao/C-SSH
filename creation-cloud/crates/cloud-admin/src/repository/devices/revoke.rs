//! 在账号与设备锁内清除绑定会话，再软撤销设备授权。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AdminDevice, model::AdminDeviceRow, repository::map_write_error};

pub(crate) const REVOKE_SQL: &str = r#"
    WITH updated AS (
        UPDATE devices
        SET revoked_at = now(), updated_at = now()
        WHERE id = $1 AND revoked_at IS NULL
        RETURNING id, account_id, name, platform, public_id, last_seen_at,
                  revoked_at, created_at, updated_at
    )
    SELECT updated.id, updated.account_id, accounts.email AS owner_email,
           updated.name, updated.platform, updated.public_id, updated.last_seen_at,
           updated.revoked_at, updated.created_at, updated.updated_at
    FROM updated
    JOIN accounts ON accounts.id = updated.account_id
"#;
pub(crate) const LOCK_ACCOUNT_SQL: &str = "SELECT id FROM accounts WHERE id = $1 FOR UPDATE";
pub(crate) const LOCK_DEVICE_SQL: &str =
    "SELECT id FROM devices WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL FOR UPDATE";
pub(crate) const DELETE_DEVICE_SESSIONS_SQL: &str =
    "DELETE FROM sessions WHERE account_id = $1 AND device_id = $2";

pub(crate) async fn execute(pool: &PgPool, device_id: Uuid) -> AppResult<AdminDevice> {
    let mut transaction = pool.begin().await.map_err(map_write_error)?;
    let account_id = sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM devices WHERE id = $1")
        .bind(device_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))?;
    let account_exists = sqlx::query_scalar::<_, Uuid>(LOCK_ACCOUNT_SQL)
        .bind(account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_write_error)?
        .is_some();
    if !account_exists {
        return Err(AppError::NotFound("设备不存在或已撤销".to_owned()));
    }
    let device_exists = sqlx::query_scalar::<_, Uuid>(LOCK_DEVICE_SQL)
        .bind(account_id)
        .bind(device_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_write_error)?
        .is_some();
    if !device_exists {
        return Err(AppError::NotFound("设备不存在或已撤销".to_owned()));
    }
    sqlx::query(DELETE_DEVICE_SESSIONS_SQL)
        .bind(account_id)
        .bind(device_id)
        .execute(&mut *transaction)
        .await
        .map_err(map_write_error)?;
    let row = sqlx::query_as::<_, AdminDeviceRow>(REVOKE_SQL)
        .bind(device_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))?;
    let device = AdminDevice::try_from(row)?;
    transaction.commit().await.map_err(map_write_error)?;
    Ok(device)
}
