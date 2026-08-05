//! 登录邮箱挑战的串行创建、一次性消费与投递状态。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::error;
use crate::verification::MAX_ATTEMPTS;

type ChallengeRow = (
    Uuid,
    Uuid,
    String,
    i64,
    Vec<u8>,
    i32,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
);

#[derive(Clone)]
pub(crate) struct LoginChallenge {
    pub id: Uuid,
    pub account_id: Uuid,
    pub email: String,
    pub credential_version: i64,
    pub code_digest: Vec<u8>,
    pub attempt_count: i32,
    pub expires_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub consumed_at: Option<DateTime<Utc>>,
}

pub(crate) struct NewLoginChallenge<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub email: &'a str,
    pub credential_version: i64,
    pub code_digest: &'a [u8],
    pub expires_at: DateTime<Utc>,
}

pub(crate) async fn find_account_id(pool: &PgPool, challenge_id: Uuid) -> AppResult<Option<Uuid>> {
    sqlx::query_scalar("SELECT account_id FROM login_verification_challenges WHERE id = $1")
        .bind(challenge_id)
        .fetch_optional(pool)
        .await
        .map_err(error::storage)
}

pub(crate) async fn lock_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<Option<LoginChallenge>> {
    sqlx::query_as::<_, ChallengeRow>(
        "SELECT id, account_id, email, credential_version, code_digest, attempt_count, \
         expires_at, sent_at, consumed_at \
         FROM login_verification_challenges WHERE id = $1 FOR UPDATE",
    )
    .bind(challenge_id)
    .fetch_optional(&mut **transaction)
    .await
    .map(|row| row.map(into_challenge))
    .map_err(error::storage)
}

pub(crate) async fn replace_open(
    transaction: &mut Transaction<'_, Postgres>,
    challenge: NewLoginChallenge<'_>,
    cooldown_seconds: i32,
) -> AppResult<()> {
    let current = sqlx::query_as::<_, (Option<DateTime<Utc>>, DateTime<Utc>)>(
        "SELECT sent_at, created_at FROM login_verification_challenges \
         WHERE account_id = $1 AND consumed_at IS NULL \
         ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(challenge.account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?;
    let last_delivery_started_at =
        current.map(|(sent_at, created_at)| sent_at.unwrap_or(created_at));
    let cooldown_seconds = i64::from(cooldown_seconds.max(1));
    if last_delivery_started_at.is_some_and(|started_at| {
        Utc::now().signed_duration_since(started_at).num_seconds() < cooldown_seconds
    }) {
        return Err(AppError::RateLimitedAfter {
            message: "登录验证码发送过于频繁，请稍后重试".to_owned(),
            retry_after_seconds: retry_after_seconds(last_delivery_started_at, cooldown_seconds),
        });
    }
    sqlx::query(
        "UPDATE login_verification_challenges SET consumed_at = now() \
         WHERE account_id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge.account_id)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    sqlx::query(
        "INSERT INTO login_verification_challenges \
         (id, account_id, email, credential_version, code_digest, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(challenge.id)
    .bind(challenge.account_id)
    .bind(challenge.email)
    .bind(challenge.credential_version)
    .bind(challenge.code_digest)
    .bind(challenge.expires_at)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn mark_sent(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE login_verification_challenges SET sent_at = now() \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn cancel_unsent(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE login_verification_challenges SET consumed_at = now() \
         WHERE id = $1 AND sent_at IS NULL AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn increment_attempt(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE login_verification_challenges \
         SET attempt_count = LEAST(attempt_count + 1, $2) WHERE id = $1",
    )
    .bind(challenge_id)
    .bind(MAX_ATTEMPTS)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn consume(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE login_verification_challenges SET consumed_at = now() \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

fn into_challenge(value: ChallengeRow) -> LoginChallenge {
    LoginChallenge {
        id: value.0,
        account_id: value.1,
        email: value.2,
        credential_version: value.3,
        code_digest: value.4,
        attempt_count: value.5,
        expires_at: value.6,
        sent_at: value.7,
        consumed_at: value.8,
    }
}

fn retry_after_seconds(
    last_delivery_started_at: Option<DateTime<Utc>>,
    cooldown_seconds: i64,
) -> u64 {
    let elapsed = last_delivery_started_at
        .map(|value| Utc::now().signed_duration_since(value).num_seconds())
        .unwrap_or_default()
        .max(0);
    cooldown_seconds
        .saturating_sub(elapsed)
        .max(1)
        .try_into()
        .unwrap_or(1)
}
