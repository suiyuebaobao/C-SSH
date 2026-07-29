//! 原子创建待验证账号、必有资料与首个邮箱挑战。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use super::{error, verification};

pub(crate) struct PendingAccount<'a> {
    pub account_id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub locale: &'a str,
    pub challenge_id: Uuid,
    pub code_digest: &'a [u8],
    pub expires_at: DateTime<Utc>,
}

pub(crate) async fn prepare(pool: &PgPool, account: PendingAccount<'_>) -> AppResult<bool> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    verification::lock_email(&mut transaction, account.email).await?;
    if let Some(existing) = verification::find_account(&mut transaction, account.email).await? {
        let should_send = existing.status == "pending_verification"
            && existing.email_verified_at.is_none()
            && verification::replace_if_cooled_down(
                &mut transaction,
                existing.id,
                account.email,
                account.challenge_id,
                account.code_digest,
                account.expires_at,
            )
            .await?;
        transaction.commit().await.map_err(error::storage)?;
        return Ok(should_send);
    }

    sqlx::query(
        "INSERT INTO accounts \
         (id, email, password_hash, status, email_verified_at) \
         VALUES ($1, $2, $3, 'pending_verification', NULL)",
    )
    .bind(account.account_id)
    .bind(account.email)
    .bind(account.password_hash)
    .execute(&mut *transaction)
    .await
    .map_err(error::create_account)?;
    sqlx::query(
        "INSERT INTO user_profiles (account_id, display_name, locale) \
         VALUES ($1, $2, $3)",
    )
    .bind(account.account_id)
    .bind(account.display_name)
    .bind(account.locale)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    verification::insert(
        &mut transaction,
        account.account_id,
        account.email,
        account.challenge_id,
        account.code_digest,
        account.expires_at,
    )
    .await?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(true)
}
