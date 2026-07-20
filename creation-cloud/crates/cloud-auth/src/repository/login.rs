//! 查询登录凭据并为通过校验的账号写入新会话。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) struct LoginAccount {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub status: String,
}

pub(crate) async fn find_account(pool: &PgPool, email: &str) -> AppResult<Option<LoginAccount>> {
    sqlx::query_as::<_, (Uuid, String, String, String, String)>(
        "SELECT id, email, password_hash, role, status FROM accounts WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|value| LoginAccount {
            id: value.0,
            email: value.1,
            password_hash: value.2,
            role: value.3,
            status: value.4,
        })
    })
    .map_err(error::storage)
}

pub(crate) async fn insert_session(
    pool: &PgPool,
    session_id: Uuid,
    account_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sessions (id, account_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(account_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}
