//! 从管理只读视图读取用户概览。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{UserOverview, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<UserOverview> {
    sqlx::query_as::<_, UserOverview>("SELECT * FROM admin_user_overview")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)
}
