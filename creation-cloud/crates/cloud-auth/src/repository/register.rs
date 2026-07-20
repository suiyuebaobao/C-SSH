//! 原子写入注册产生的账号、用户资料和首个 Web 会话。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::error;

pub(crate) struct NewAccount<'a> {
    pub account_id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub locale: &'a str,
    pub session_id: Uuid,
    pub token_hash: &'a [u8],
    pub expires_at: DateTime<Utc>,
}

pub(crate) async fn insert(pool: &PgPool, account: NewAccount<'_>) -> AppResult<()> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    sqlx::query("INSERT INTO accounts (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(account.account_id)
        .bind(account.email)
        .bind(account.password_hash)
        .execute(&mut *transaction)
        .await
        .map_err(error::create_account)?;

    sqlx::query("INSERT INTO user_profiles (account_id, display_name, locale) VALUES ($1, $2, $3)")
        .bind(account.account_id)
        .bind(account.display_name)
        .bind(account.locale)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;

    sqlx::query(
        "INSERT INTO sessions (id, account_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(account.session_id)
    .bind(account.account_id)
    .bind(account.token_hash)
    .bind(account.expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;

    transaction.commit().await.map_err(error::storage)
}
