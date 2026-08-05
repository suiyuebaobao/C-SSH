//! 为账号资料更新提供管理员行锁、原子写入与结果投影。

use cloud_domain::{AppError, AppResult};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::{AdminUser, model::AdminUserRow, repository::map_write_error};

#[derive(Debug, FromRow)]
pub(crate) struct LockedAccount {
    pub id: Uuid,
    pub email: Option<String>,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
}

pub(crate) struct UserUpdate<'a> {
    pub email: Option<&'a str>,
    pub display_name: Option<&'a str>,
    pub admin_login_name: Option<&'a str>,
    pub clear_admin_login_name: bool,
    pub role: Option<&'a str>,
    pub status: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}

pub(crate) const LOCK_ACTIVE_ADMINS_SQL: &str =
    "SELECT id FROM accounts WHERE role = 'admin' AND status = 'active' ORDER BY id FOR UPDATE";
pub(crate) const LOCK_ACCOUNT_SQL: &str = r#"
    SELECT accounts.id, accounts.email, accounts.admin_login_name,
           accounts.role, accounts.status,
           accounts.email_verified_at IS NOT NULL AS email_verified
    FROM accounts
    WHERE accounts.id = $1
    FOR UPDATE
"#;
pub(crate) const APPLY_UPDATE_SQL: &str = r#"
    UPDATE accounts
    SET email = COALESCE($2, email),
        admin_login_name = CASE
            WHEN COALESCE($5, role) = 'user' OR $4 THEN NULL
            WHEN $3 IS NOT NULL THEN $3
            ELSE admin_login_name
        END,
        role = COALESCE($5, role),
        status = COALESCE($6, status),
        password_hash = COALESCE($7, password_hash),
        credential_version = credential_version + CASE WHEN $7 IS NULL THEN 0 ELSE 1 END,
        email_verified_at = CASE
            WHEN $2 IS NULL THEN email_verified_at
            WHEN COALESCE($6, status) = 'pending_verification' THEN NULL
            ELSE now()
        END,
        updated_at = now()
    WHERE id = $1
"#;
pub(crate) const APPLY_PROFILE_SQL: &str = r#"
    UPDATE user_profiles SET display_name = $2, updated_at = now() WHERE account_id = $1
"#;
pub(crate) const GET_UPDATED_SQL: &str = r#"
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
    input: UserUpdate<'_>,
) -> AppResult<AdminUser> {
    sqlx::query(APPLY_UPDATE_SQL)
        .bind(account_id)
        .bind(input.email)
        .bind(input.admin_login_name)
        .bind(input.clear_admin_login_name)
        .bind(input.role)
        .bind(input.status)
        .bind(input.password_hash)
        .execute(&mut **transaction)
        .await
        .map_err(super::create::map_account_write_error)?;
    if let Some(display_name) = input.display_name {
        sqlx::query(APPLY_PROFILE_SQL)
            .bind(account_id)
            .bind(display_name)
            .execute(&mut **transaction)
            .await
            .map_err(map_write_error)?;
    }
    let row = sqlx::query_as::<_, AdminUserRow>(GET_UPDATED_SQL)
        .bind(account_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    AdminUser::try_from(row)
}
