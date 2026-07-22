//! 为角色和状态变更提供同一事务内的管理员行锁、账号写入与会话清理。

use cloud_domain::{AppError, AppResult};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::{AdminUpdateUserInput, AdminUser, model::AdminUserRow, repository::map_write_error};

#[derive(Debug, FromRow)]
pub(crate) struct LockedAccount {
    pub id: Uuid,
    pub role: String,
    pub status: String,
}

pub(crate) const LOCK_ACTIVE_ADMINS_SQL: &str =
    "SELECT id FROM accounts WHERE role = 'admin' AND status = 'active' ORDER BY id FOR UPDATE";
pub(crate) const LOCK_ACCOUNT_SQL: &str =
    "SELECT id, role, status FROM accounts WHERE id = $1 FOR UPDATE";
pub(crate) const APPLY_UPDATE_SQL: &str = r#"
    WITH updated AS (
        UPDATE accounts
        SET role = COALESCE($2, role),
            status = COALESCE($3, status),
            admin_login_name = CASE
                WHEN COALESCE($2, role) = 'user' THEN NULL
                ELSE admin_login_name
            END,
            updated_at = now()
        WHERE id = $1
        RETURNING id, email, role, status, created_at, updated_at
    )
    SELECT updated.id, updated.email,
           COALESCE(user_profiles.display_name, '') AS display_name,
           updated.role, updated.status,
           (SELECT count(*)::BIGINT FROM devices WHERE devices.account_id = updated.id)
               AS device_count,
           updated.created_at, updated.updated_at
    FROM updated
    LEFT JOIN user_profiles ON user_profiles.account_id = updated.id
"#;
pub(crate) async fn lock_active_admins(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(LOCK_ACTIVE_ADMINS_SQL)
        .fetch_all(&mut **transaction)
        .await
        .map_err(map_write_error)
}

pub(crate) async fn lock_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<LockedAccount> {
    sqlx::query_as::<_, LockedAccount>(LOCK_ACCOUNT_SQL)
        .bind(account_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(map_write_error)?
        .ok_or_else(|| AppError::NotFound("账号不存在".to_owned()))
}

pub(crate) async fn apply(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    input: AdminUpdateUserInput,
) -> AppResult<AdminUser> {
    let row = sqlx::query_as::<_, AdminUserRow>(APPLY_UPDATE_SQL)
        .bind(account_id)
        .bind(input.role.map(|value| value.as_str()))
        .bind(input.status.map(|value| value.as_str()))
        .fetch_one(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    AdminUser::try_from(row)
}
