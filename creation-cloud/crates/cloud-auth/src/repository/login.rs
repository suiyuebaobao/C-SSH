//! 查询登录凭据并为通过校验的账号写入新会话。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use super::error;

type LoginAccountRow = (Uuid, String, Option<String>, String, String, String);

#[derive(Clone)]
pub(crate) struct LoginAccount {
    pub id: Uuid,
    pub email: String,
    pub admin_login_name: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub status: String,
}

pub(crate) const FIND_BY_EMAIL_SQL: &str = r#"
    SELECT id, email, admin_login_name, password_hash, role, status
    FROM accounts
    WHERE email = $1
"#;

pub(crate) const FIND_ADMIN_BY_LOGIN_NAME_SQL: &str = r#"
    SELECT id, email, admin_login_name, password_hash, role, status
    FROM accounts
    WHERE admin_login_name = $1
      AND role = 'admin'
      AND status = 'active'
"#;

pub(crate) const LOCK_ACCOUNT_BY_ID_SQL: &str = r#"
    SELECT id, email, admin_login_name, password_hash, role, status
    FROM accounts
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) const INSERT_SESSION_SQL: &str =
    "INSERT INTO sessions (id, account_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)";

pub(crate) async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<LoginAccount>> {
    find(pool, FIND_BY_EMAIL_SQL, email).await
}

pub(crate) async fn find_admin_by_login_name(
    pool: &PgPool,
    login_name: &str,
) -> AppResult<Option<LoginAccount>> {
    find(pool, FIND_ADMIN_BY_LOGIN_NAME_SQL, login_name).await
}

async fn find(pool: &PgPool, sql: &str, identifier: &str) -> AppResult<Option<LoginAccount>> {
    sqlx::query_as::<_, LoginAccountRow>(sql)
        .bind(identifier)
        .fetch_optional(pool)
        .await
        .map(into_account)
        .map_err(error::storage)
}

pub(crate) async fn lock_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<Option<LoginAccount>> {
    sqlx::query_as::<_, LoginAccountRow>(LOCK_ACCOUNT_BY_ID_SQL)
        .bind(account_id)
        .fetch_optional(&mut **transaction)
        .await
        .map(into_account)
        .map_err(error::storage)
}

fn into_account(row: Option<LoginAccountRow>) -> Option<LoginAccount> {
    row.map(|value| LoginAccount {
        id: value.0,
        email: value.1,
        admin_login_name: value.2,
        password_hash: value.3,
        role: value.4,
        status: value.5,
    })
}

pub(crate) async fn insert_session(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    account_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(INSERT_SESSION_SQL)
        .bind(session_id)
        .bind(account_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
    Ok(())
}
