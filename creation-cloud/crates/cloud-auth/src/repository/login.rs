//! 查询登录凭据并为通过校验的账号写入新会话。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use super::error;

type LoginAccountRow = (
    Uuid,
    Option<String>,
    Option<DateTime<Utc>>,
    Option<String>,
    String,
    String,
    String,
    i64,
    i32,
    Option<DateTime<Utc>>,
);

#[derive(Clone)]
pub(crate) struct LoginAccount {
    pub id: Uuid,
    pub email: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub admin_login_name: Option<String>,
    pub password_hash: String,
    pub role: String,
    pub status: String,
    pub credential_version: i64,
    pub consecutive_login_failures: i32,
    pub login_locked_until: Option<DateTime<Utc>>,
}

pub(crate) const FIND_BY_EMAIL_SQL: &str = r#"
    SELECT id, email, email_verified_at, admin_login_name, password_hash,
           role, status, credential_version, consecutive_login_failures,
           login_locked_until
    FROM accounts
    WHERE email = $1
"#;

pub(crate) const FIND_ADMIN_BY_LOGIN_NAME_SQL: &str = r#"
    SELECT id, email, email_verified_at, admin_login_name, password_hash,
           role, status, credential_version, consecutive_login_failures,
           login_locked_until
    FROM accounts
    WHERE admin_login_name = $1
      AND role = 'admin'
      AND status = 'active'
"#;

pub(crate) const LOCK_ACCOUNT_BY_ID_SQL: &str = r#"
    SELECT id, email, email_verified_at, admin_login_name, password_hash,
           role, status, credential_version, consecutive_login_failures,
           login_locked_until
    FROM accounts
    WHERE id = $1
    FOR UPDATE
"#;

pub(crate) const INSERT_SESSION_SQL: &str = "INSERT INTO sessions \
     (id, account_id, token_hash, expires_at, absolute_expires_at, credential_version, \
      session_kind, last_login_ip, user_agent) \
     VALUES ($1, $2, $3, $4, $4, $5, 'unbound', $6::inet, $7)";

pub(crate) const UPDATE_LOGIN_FAILURES_SQL: &str = "UPDATE accounts SET \
     consecutive_login_failures = $2, login_locked_until = $3 \
     WHERE id = $1";

pub(crate) const CLEAR_LOGIN_FAILURES_SQL: &str = "UPDATE accounts SET \
     consecutive_login_failures = 0, login_locked_until = NULL \
     WHERE id = $1";

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
        email_verified_at: value.2,
        admin_login_name: value.3,
        password_hash: value.4,
        role: value.5,
        status: value.6,
        credential_version: value.7,
        consecutive_login_failures: value.8,
        login_locked_until: value.9,
    })
}

pub(crate) async fn update_login_failures(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    consecutive_failures: i32,
    locked_until: Option<DateTime<Utc>>,
) -> AppResult<()> {
    sqlx::query(UPDATE_LOGIN_FAILURES_SQL)
        .bind(account_id)
        .bind(consecutive_failures)
        .bind(locked_until)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn clear_login_failures(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(CLEAR_LOGIN_FAILURES_SQL)
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn insert_session(
    transaction: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    account_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
    credential_version: i64,
    metadata: &crate::TrustedRequestMetadata,
) -> AppResult<()> {
    sqlx::query(INSERT_SESSION_SQL)
        .bind(session_id)
        .bind(account_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(credential_version)
        .bind(&metadata.last_login_ip)
        .bind(&metadata.user_agent)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
    Ok(())
}
