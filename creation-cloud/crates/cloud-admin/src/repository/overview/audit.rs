//! 从管理只读视图读取审计概览。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{SecurityAuditOverview, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool) -> AppResult<SecurityAuditOverview> {
    sqlx::query_as::<_, SecurityAuditOverview>("SELECT * FROM admin_audit_overview")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)
}
