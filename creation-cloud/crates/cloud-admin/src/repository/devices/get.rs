//! 按设备标识读取包含脱敏账号归属的管理设备投影。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AdminDevice, model::AdminDeviceRow, repository::map_read_error};

pub(crate) const GET_SQL: &str = r#"
    SELECT devices.id, devices.account_id, accounts.email AS owner_email,
           devices.name, devices.platform, devices.public_id, devices.last_seen_at,
           devices.revoked_at, devices.created_at, devices.updated_at
    FROM devices
    JOIN accounts ON accounts.id = devices.account_id
    WHERE devices.id = $1
"#;

pub(crate) async fn execute(pool: &PgPool, device_id: Uuid) -> AppResult<AdminDevice> {
    let row = sqlx::query_as::<_, AdminDeviceRow>(GET_SQL)
        .bind(device_id)
        .fetch_optional(pool)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_owned()))?;
    AdminDevice::try_from(row)
}
