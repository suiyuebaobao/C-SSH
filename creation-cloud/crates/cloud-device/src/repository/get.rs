//! 按账号所有权与设备主键读取单个设备。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Device, model::DeviceRow};

use super::error;

pub(crate) const FIND_SQL: &str = "SELECT id, account_id, name, platform, public_id, last_seen_at, \
     revoked_at, created_at, updated_at \
     FROM devices WHERE account_id = $1 AND id = $2";

pub(crate) async fn find(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
) -> AppResult<Option<Device>> {
    sqlx::query_as::<_, DeviceRow>(FIND_SQL)
        .bind(account_id)
        .bind(device_id)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Device::from_row))
        .map_err(error::storage)
}
