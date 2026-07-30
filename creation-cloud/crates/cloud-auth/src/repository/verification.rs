//! 邮箱挑战的串行创建、投递状态与一次性消费。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::error;
use crate::{
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    token,
    verification::{MAX_ATTEMPTS, RESEND_COOLDOWN_SECONDS},
};

pub(crate) struct AccountState {
    pub id: Uuid,
    pub status: String,
    pub email_verified_at: Option<DateTime<Utc>>,
}

type ChallengeRow = (
    Uuid,
    Uuid,
    Vec<u8>,
    i32,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

pub(crate) async fn lock_email(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(email)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn find_account(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str,
) -> AppResult<Option<AccountState>> {
    sqlx::query_as::<_, (Uuid, String, Option<DateTime<Utc>>)>(
        "SELECT id, status, email_verified_at FROM accounts \
         WHERE email = $1 FOR UPDATE",
    )
    .bind(email)
    .fetch_optional(&mut **transaction)
    .await
    .map(|row| {
        row.map(|value| AccountState {
            id: value.0,
            status: value.1,
            email_verified_at: value.2,
        })
    })
    .map_err(error::storage)
}

pub(crate) async fn replace_if_cooled_down(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    email: &str,
    challenge_id: Uuid,
    code_digest: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<bool> {
    let latest = sqlx::query_as::<_, (DateTime<Utc>, Option<DateTime<Utc>>)>(
        "SELECT created_at, sent_at FROM email_verification_challenges \
         WHERE account_id = $1 AND consumed_at IS NULL \
         ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?;
    if latest
        .and_then(|(_, sent_at)| sent_at)
        .is_some_and(|sent_at| {
            Utc::now().signed_duration_since(sent_at).num_seconds() < RESEND_COOLDOWN_SECONDS
        })
    {
        return Ok(false);
    }
    consume_open(transaction, account_id).await?;
    insert(
        transaction,
        account_id,
        email,
        challenge_id,
        code_digest,
        expires_at,
    )
    .await?;
    Ok(true)
}

pub(crate) async fn insert(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    email: &str,
    challenge_id: Uuid,
    code_digest: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO email_verification_challenges \
         (id, account_id, email, code_digest, expires_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(challenge_id)
    .bind(account_id)
    .bind(email)
    .bind(code_digest)
    .bind(expires_at)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn prepare_resend(
    pool: &PgPool,
    verification_available: bool,
    email: &str,
    challenge_id: Uuid,
    code_digest: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<bool> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let verification_enabled =
        super::settings::email_verification_enabled(&mut transaction).await?;
    if !verification_enabled {
        transaction.commit().await.map_err(error::storage)?;
        return Ok(false);
    }
    if !verification_available {
        return Err(AppError::Unavailable("邮箱验证密钥尚未安全配置".to_owned()));
    }
    lock_email(&mut transaction, email).await?;
    let Some(account) = find_account(&mut transaction, email).await? else {
        transaction.commit().await.map_err(error::storage)?;
        return Ok(false);
    };
    let should_send = account.status == "pending_verification"
        && account.email_verified_at.is_none()
        && replace_if_cooled_down(
            &mut transaction,
            account.id,
            email,
            challenge_id,
            code_digest,
            expires_at,
        )
        .await?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(should_send)
}

pub(crate) async fn mark_sent(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE email_verification_challenges SET sent_at = now() \
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
        "UPDATE email_verification_challenges SET consumed_at = now() \
         WHERE id = $1 AND sent_at IS NULL AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn verify_and_issue(
    pool: &PgPool,
    email: &str,
    supplied_digest: impl FnOnce(Uuid) -> [u8; 32],
    expires_at: DateTime<Utc>,
    session_id: Uuid,
    raw_token: String,
    token_hash: Vec<u8>,
) -> AppResult<IssuedSession> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    lock_email(&mut transaction, email).await?;
    let account = find_account(&mut transaction, email)
        .await?
        .filter(|account| {
            account.status == "pending_verification" && account.email_verified_at.is_none()
        })
        .ok_or_else(invalid_code)?;
    let challenge = sqlx::query_as::<_, ChallengeRow>(
        "SELECT id, account_id, code_digest, attempt_count, expires_at, consumed_at \
         FROM email_verification_challenges \
         WHERE account_id = $1 AND email = $2 AND consumed_at IS NULL \
         ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(account.id)
    .bind(email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?
    .ok_or_else(invalid_code)?;
    let actual = supplied_digest(challenge.0);
    let valid = challenge.1 == account.id
        && challenge.3 < MAX_ATTEMPTS
        && challenge.4 > Utc::now()
        && challenge.5.is_none()
        && crate::verification::matches(&challenge.2, &actual);
    if !valid {
        sqlx::query(
            "UPDATE email_verification_challenges \
             SET attempt_count = LEAST(attempt_count + 1, $2) WHERE id = $1",
        )
        .bind(challenge.0)
        .bind(MAX_ATTEMPTS)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
        transaction.commit().await.map_err(error::storage)?;
        return Err(invalid_code());
    }
    sqlx::query(
        "UPDATE email_verification_challenges \
         SET consumed_at = now() WHERE account_id = $1 AND consumed_at IS NULL",
    )
    .bind(account.id)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let credential_version = sqlx::query_scalar::<_, i64>(
        "UPDATE accounts SET status = 'active', email_verified_at = now(), \
         updated_at = now() WHERE id = $1 AND status = 'pending_verification' \
         RETURNING credential_version",
    )
    .bind(account.id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::storage)?;
    sqlx::query(
        "INSERT INTO sessions \
         (id, account_id, token_hash, credential_version, session_kind, \
          expires_at, absolute_expires_at) \
         VALUES ($1, $2, $3, $4, 'unbound', $5, $5)",
    )
    .bind(session_id)
    .bind(account.id)
    .bind(token_hash)
    .bind(credential_version)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(IssuedSession {
        raw_token: raw_token.clone(),
        session: AuthenticatedSession {
            session_id,
            account_id: account.id,
            email: email.to_owned(),
            admin_login_name: None,
            role: "user".to_owned(),
            device_id: None,
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        metadata: SessionMetadata::unbound(expires_at, true),
    })
}

pub(crate) async fn consume_open(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE email_verification_challenges SET consumed_at = now() \
         WHERE account_id = $1 AND consumed_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

fn invalid_code() -> AppError {
    AppError::Unauthorized("验证码无效、已过期或已达到尝试上限".to_owned())
}
