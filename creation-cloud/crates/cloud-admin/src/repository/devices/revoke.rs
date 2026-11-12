//! 在账号与设备锁内清除绑定会话，再软撤销设备授权。

use cloud_domain::{AppError, AppResult};
use cloud_notification::{record_account_event, AccountNotificationEvent};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{model::AdminDeviceRow, repository::map_write_error, AdminDevice};

pub(crate) const REVOKE_SQL: &str = r#"
    WITH updated AS (
        UPDATE devices
        SET revoked_at = now(), updated_at = now()
        WHERE id = ANY($1) AND account_id = $3 AND revoked_at IS NULL
        RETURNING id, account_id, name, platform, public_id, last_seen_at,
                  revoked_at, created_at, updated_at, physical_device_fingerprint
    )
    SELECT updated.id, updated.account_id, accounts.email AS owner_email,
           accounts.admin_login_name AS owner_admin_login_name,
           updated.name, updated.platform, updated.public_id, updated.last_seen_at,
           updated.revoked_at, updated.created_at, updated.updated_at, updated.physical_device_fingerprint
    FROM updated
    JOIN accounts ON accounts.id = updated.account_id
    WHERE updated.id = $2
"#;
pub(crate) const LOCK_ACCOUNT_SQL: &str = "SELECT id FROM accounts WHERE id = $1 FOR UPDATE";
pub(crate) const LOCK_DEVICE_SQL: &str =
    "SELECT d.id FROM devices d JOIN device_identity_groups g ON d.id=ANY(g.registration_ids) \
     JOIN devices chosen ON chosen.id=$2 AND chosen.account_id=$1 AND chosen.revoked_at IS NULL \
     WHERE g.account_id=$1 AND d.account_id=$1 AND chosen.id=ANY(g.registration_ids) \
     AND d.revoked_at IS NULL ORDER BY d.id FOR UPDATE OF d";
pub(crate) const DELETE_DEVICE_SESSIONS_SQL: &str =
    "DELETE FROM sessions WHERE account_id = $1 AND device_id = ANY($2)";

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
    let members = sqlx::query_scalar::<_, Uuid>(LOCK_DEVICE_SQL)
        .bind(account_id)
        .bind(device_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(map_write_error)?;
    if members.is_empty() {
        return Err(AppError::NotFound("设备不存在或已撤销".to_owned()));
    }
    sqlx::query(DELETE_DEVICE_SESSIONS_SQL)
        .bind(account_id)
        .bind(&members)
        .execute(&mut *transaction)
        .await
        .map_err(map_write_error)?;
    for query in [
        "DELETE FROM cloud_sync_resource_deliveries WHERE account_id=$1 AND device_id=ANY($2)",
        "DELETE FROM cloud_sync_pull_watermarks WHERE account_id=$1 AND device_id=ANY($2)",
    ] {
        sqlx::query(query)
            .bind(account_id)
            .bind(&members)
            .execute(&mut *transaction)
            .await
            .map_err(map_write_error)?;
    }
    let row = sqlx::query_as::<_, AdminDeviceRow>(REVOKE_SQL)
        .bind(&members)
        .bind(device_id)
        .bind(account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))?;
    let device = AdminDevice::try_from(row)?;
    record_account_event(
        &mut transaction,
        account_id,
        AccountNotificationEvent::DeviceRevoked { device_id },
    )
    .await?;
    transaction.commit().await.map_err(map_write_error)?;
    Ok(device)
}
