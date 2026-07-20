//! 按账号、平台和撤销状态分页读取管理设备投影。

use cloud_domain::{AppResult, Page};
use cloud_store::PgPool;
use sqlx::FromRow;

use crate::{
    AdminDevice, model::AdminDeviceListFilter, model::AdminDeviceRow, repository::map_read_error,
};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

pub(crate) const COUNT_SQL: &str = r#"
    SELECT count(*)::BIGINT AS total
    FROM devices
    WHERE ($1::UUID IS NULL OR account_id = $1)
      AND ($2::TEXT IS NULL OR platform = $2)
      AND ($3::BOOLEAN IS NULL OR (revoked_at IS NOT NULL) = $3)
"#;

pub(crate) const LIST_SQL: &str = r#"
    SELECT devices.id, devices.account_id, accounts.email AS owner_email,
           devices.name, devices.platform, devices.public_id, devices.last_seen_at,
           devices.revoked_at, devices.created_at, devices.updated_at
    FROM devices
    JOIN accounts ON accounts.id = devices.account_id
    WHERE ($1::UUID IS NULL OR devices.account_id = $1)
      AND ($2::TEXT IS NULL OR devices.platform = $2)
      AND ($3::BOOLEAN IS NULL OR (devices.revoked_at IS NOT NULL) = $3)
    ORDER BY devices.created_at DESC, devices.id DESC
    LIMIT $4 OFFSET $5
"#;

pub(crate) async fn execute(
    pool: &PgPool,
    filter: &AdminDeviceListFilter,
) -> AppResult<Page<AdminDevice>> {
    let platform = filter.platform.map(|value| value.as_str());
    let count = sqlx::query_as::<_, CountRow>(COUNT_SQL)
        .bind(filter.account_id)
        .bind(platform)
        .bind(filter.revoked)
        .fetch_one(pool)
        .await
        .map_err(map_read_error)?;
    let rows = sqlx::query_as::<_, AdminDeviceRow>(LIST_SQL)
        .bind(filter.account_id)
        .bind(platform)
        .bind(filter.revoked)
        .bind(i64::from(filter.page.size))
        .bind(filter.page.offset())
        .fetch_all(pool)
        .await
        .map_err(map_read_error)?;
    let items = rows
        .into_iter()
        .map(AdminDevice::try_from)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: filter.page.page,
        size: filter.page.size,
        total: count.total,
    })
}
