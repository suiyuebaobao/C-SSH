//! 在管理用户事务中创建已核验账号及其资料，并返回最小暴露投影。

use cloud_domain::{AppError, AppResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{AdminUser, model::AdminUserRow, repository::map_write_error};

pub(crate) struct NewUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub role: &'a str,
    pub status: &'a str,
    pub admin_login_name: Option<&'a str>,
}

pub(crate) const INSERT_ACCOUNT_SQL: &str = r#"
    INSERT INTO accounts (
        id, email, admin_login_name, password_hash, role, status,
        email_verified_at, credential_version
    )
    VALUES ($1, $2, $3, $4, $5, $6, now(), 1)
"#;

pub(crate) const INSERT_PROFILE_SQL: &str = r#"
    INSERT INTO user_profiles (account_id, display_name, locale)
    VALUES ($1, $2, 'zh-CN')
"#;

pub(crate) const GET_CREATED_SQL: &str = r#"
    SELECT accounts.id, accounts.email, accounts.admin_login_name,
           accounts.email_verified_at IS NOT NULL AS email_verified,
           user_profiles.display_name, accounts.role, accounts.status,
           0::BIGINT AS device_count, 0::BIGINT AS host_count,
           accounts.created_at, accounts.updated_at
    FROM accounts
    JOIN user_profiles ON user_profiles.account_id = accounts.id
    WHERE accounts.id = $1
"#;

pub(crate) async fn execute(
    transaction: &mut Transaction<'_, Postgres>,
    input: NewUser<'_>,
) -> AppResult<AdminUser> {
    sqlx::query(INSERT_ACCOUNT_SQL)
        .bind(input.id)
        .bind(input.email)
        .bind(input.admin_login_name)
        .bind(input.password_hash)
        .bind(input.role)
        .bind(input.status)
        .execute(&mut **transaction)
        .await
        .map_err(map_account_write_error)?;
    sqlx::query(INSERT_PROFILE_SQL)
        .bind(input.id)
        .bind(input.display_name)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    let row = sqlx::query_as::<_, AdminUserRow>(GET_CREATED_SQL)
        .bind(input.id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    AdminUser::try_from(row)
}

pub(crate) fn map_account_write_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("邮箱或管理员登录名不可用".to_owned())
    } else {
        map_write_error(error)
    }
}
