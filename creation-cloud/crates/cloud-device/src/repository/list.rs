//! 按当前账号范围分页查询设备及总数。

use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{model::DeviceRow, Device};

use super::error;

pub(crate) const LIST_SQL: &str =
    "SELECT d.id, d.account_id, d.name, d.platform, d.public_id, d.last_seen_at, \
     d.revoked_at, d.created_at, d.updated_at, d.physical_device_fingerprint \
     FROM device_identity_groups g JOIN devices d ON d.id = \
         CASE WHEN $4::UUID = ANY(g.registration_ids) THEN $4 ELSE g.representative_id END \
     WHERE g.account_id = $1 ORDER BY d.created_at DESC, d.id LIMIT $2 OFFSET $3";

pub(crate) async fn find(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
    current_device: Option<Uuid>,
) -> AppResult<(Vec<Device>, i64)> {
    let items = sqlx::query_as::<_, DeviceRow>(LIST_SQL)
        .bind(account_id)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .bind(current_device)
        .fetch_all(pool)
        .await
        .map_err(error::storage)?;

    let count = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) AS total FROM device_identity_groups WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(error::storage)?;
    Ok((items.into_iter().map(Device::from_row).collect(), count.0))
}
