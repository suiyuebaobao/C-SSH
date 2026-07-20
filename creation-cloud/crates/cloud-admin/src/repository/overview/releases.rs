//! 从管理只读视图读取版本概览。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{ReleaseOverview, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<ReleaseOverview> {
    sqlx::query_as::<_, ReleaseOverview>("SELECT * FROM admin_release_overview")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)
}
