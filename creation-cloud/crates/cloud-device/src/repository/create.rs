//! 插入当前账号的新设备并返回数据库权威值。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Device, model::DeviceRow};

use super::error;

pub(crate) async fn insert(
    pool: &PgPool,
    device_id: Uuid,
    account_id: Uuid,
    name: &str,
    platform: &str,
    public_id: &str,
) -> AppResult<Device> {
    sqlx::query_as::<_, DeviceRow>(
        "INSERT INTO devices (id, account_id, name, platform, public_id) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, account_id, name, platform, public_id, last_seen_at, \
                   revoked_at, created_at, updated_at",
    )
    .bind(device_id)
    .bind(account_id)
    .bind(name)
    .bind(platform)
    .bind(public_id)
    .fetch_one(pool)
    .await
    .map(Device::from_row)
    .map_err(error::create)
}
