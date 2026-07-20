//! 重命名当前账号尚未撤销的单个设备。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Device, model::DeviceRow};

use super::error;

pub(crate) const RENAME_SQL: &str = "UPDATE devices SET name = $3, updated_at = now() \
     WHERE account_id = $1 AND id = $2 AND revoked_at IS NULL \
     RETURNING id, account_id, name, platform, public_id, last_seen_at, \
               revoked_at, created_at, updated_at";

pub(crate) async fn rename(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
    name: &str,
) -> AppResult<Option<Device>> {
    sqlx::query_as::<_, DeviceRow>(RENAME_SQL)
        .bind(account_id)
        .bind(device_id)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Device::from_row))
        .map_err(error::storage)
}
