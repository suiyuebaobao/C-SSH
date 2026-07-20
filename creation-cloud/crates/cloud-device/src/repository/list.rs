//! 按当前账号范围分页查询设备及总数。

use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Device, model::DeviceRow};

use super::error;

pub(crate) const LIST_SQL: &str = "SELECT id, account_id, name, platform, public_id, last_seen_at, \
     revoked_at, created_at, updated_at \
     FROM devices WHERE account_id = $1 \
     ORDER BY created_at DESC, id LIMIT $2 OFFSET $3";

pub(crate) async fn find(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<Device>, i64)> {
    let items = sqlx::query_as::<_, DeviceRow>(LIST_SQL)
        .bind(account_id)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(error::storage)?;

    let count =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) AS total FROM devices WHERE account_id = $1")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .map_err(error::storage)?;
    Ok((items.into_iter().map(Device::from_row).collect(), count.0))
}
