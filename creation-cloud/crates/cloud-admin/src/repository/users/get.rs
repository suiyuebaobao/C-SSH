//! 按账号标识读取单个管理用户投影，原文邮箱只停留在仓储内部行模型。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AdminUser, model::AdminUserRow, repository::map_read_error};

pub(crate) const GET_SQL: &str = r#"
    SELECT accounts.id, accounts.email, accounts.admin_login_name,
           accounts.email_verified_at IS NOT NULL AS email_verified,
           COALESCE(user_profiles.display_name, '') AS display_name,
           accounts.role, accounts.status,
           (SELECT count(*)::BIGINT FROM devices WHERE devices.account_id = accounts.id)
               AS device_count,
           (SELECT count(*)::BIGINT FROM cloud_hosts
            WHERE cloud_hosts.account_id = accounts.id
              AND cloud_hosts.is_deleted = FALSE) AS host_count,
           accounts.created_at, accounts.updated_at
    FROM accounts
    LEFT JOIN user_profiles ON user_profiles.account_id = accounts.id
    WHERE accounts.id = $1
"#;

pub(crate) async fn execute(pool: &PgPool, account_id: Uuid) -> AppResult<AdminUser> {
    let row = sqlx::query_as::<_, AdminUserRow>(GET_SQL)
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(map_read_error)?
        .ok_or_else(|| AppError::NotFound("账号不存在".to_owned()))?;
    AdminUser::try_from(row)
}
