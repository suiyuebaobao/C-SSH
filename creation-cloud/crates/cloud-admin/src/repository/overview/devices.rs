//! 从管理只读视图读取设备概览。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{DeviceOverview, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<DeviceOverview> {
    sqlx::query_as::<_, DeviceOverview>("SELECT * FROM admin_device_overview")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)
}
