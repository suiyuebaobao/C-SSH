//! 以单条原子更新撤销设备授权，并保留设备和用户其它数据。

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

pub(crate) async fn execute(pool: &PgPool, device_id: Uuid) -> AppResult<AdminDevice> {
    let row = sqlx::query_as::<_, AdminDeviceRow>(REVOKE_SQL)
        .bind(device_id)
        .fetch_optional(pool)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))?;
    AdminDevice::try_from(row)
}
