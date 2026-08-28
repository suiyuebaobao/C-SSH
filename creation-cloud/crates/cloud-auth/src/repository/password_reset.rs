//! Stores password-reset challenges and atomically settles a verified reset.

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_notification::{AccountNotificationEvent, record_account_event};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::{captcha, error, settings, verification};
use crate::{
    captcha::CaptchaPurpose,
    verification::{self as code_verification, MAX_ATTEMPTS},
};

type AccountRow = (
    Uuid,
    Option<String>,
    String,
    String,
    Option<DateTime<Utc>>,
    i64,
    String,
);
type OpenChallengeRow = (Uuid, DateTime<Utc>, Option<DateTime<Utc>>, DateTime<Utc>);
type SnapshotRow = (
    Uuid,
    Uuid,
    String,
    i64,
    Vec<u8>,
    i32,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    String,
    String,
    Option<DateTime<Utc>>,
    i64,
    Option<String>,
    String,
);

#[derive(Clone)]
pub(crate) struct PasswordResetSnapshot {
    pub id: Uuid,
    pub account_id: Uuid,
    pub email: String,
    pub credential_version: i64,
    pub code_digest: Vec<u8>,
    pub attempt_count: i32,
    pub expires_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub role: String,
    pub status: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub account_credential_version: i64,
    pub account_email: Option<String>,
    pub account_password_hash: String,
}

pub(crate) struct PasswordResetPreparation {
    pub delivery: Option<PasswordResetDelivery>,
}

pub(crate) struct PasswordResetDelivery {
    pub challenge_id: Uuid,
    pub email: String,
}

pub(crate) struct PreparePasswordReset<'a> {
    pub email: &'a str,
    pub challenge_id: Uuid,
    pub code: &'a str,
    pub verification_key: &'a [u8],
    pub expires_at: DateTime<Utc>,
    pub captcha_id: Uuid,
    pub captcha_digest: &'a [u8; 32],
    pub captcha_code_valid: bool,
}

pub(crate) async fn prepare(
    pool: &PgPool,
    input: PreparePasswordReset<'_>,
) -> AppResult<PasswordResetPreparation> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let auth_settings = settings::lock(&mut transaction).await?;
    if auth_settings.user_captcha_enabled {
        let valid = captcha::consume(
            &mut transaction,
            input.captcha_id,
            CaptchaPurpose::PasswordReset,
            input.captcha_digest,
        )
        .await?;
        if !valid || !input.captcha_code_valid {
            transaction.commit().await.map_err(error::storage)?;
            return Err(AppError::Unauthorized("图形验证码错误或已失效".to_owned()));
        }
    }

    verification::lock_email(&mut transaction, input.email).await?;
    let account = sqlx::query_as::<_, AccountRow>(
        "SELECT id, email, role, status, email_verified_at, credential_version, password_hash \
         FROM accounts WHERE email = $1 FOR UPDATE",
    )
    .bind(input.email)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let Some((
        account_id,
        account_email,
        role,
        status,
        email_verified_at,
        credential_version,
        _password_hash,
    )) = account
    else {
        transaction.commit().await.map_err(error::storage)?;
        return Ok(ineligible());
    };
    if account_email.as_deref() != Some(input.email)
        || !matches!(role.as_str(), "user" | "admin")
        || status != "active"
        || email_verified_at.is_none()
    {
        transaction.commit().await.map_err(error::storage)?;
        return Ok(ineligible());
    }

    let current = sqlx::query_as::<_, OpenChallengeRow>(
        "SELECT id, expires_at, sent_at, created_at \
         FROM password_reset_challenges \
         WHERE account_id = $1 AND consumed_at IS NULL \
         ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if let Some((_challenge_id, expires_at, sent_at, _created_at)) = current {
        let now = Utc::now();
        let cooldown = chrono::Duration::seconds(i64::from(auth_settings.email_cooldown_seconds));
        if expires_at > now
            && sent_at.is_some_and(|sent_at| {
                sent_at
                    .checked_add_signed(cooldown)
                    .is_some_and(|retry_at| retry_at > now)
            })
        {
            transaction.commit().await.map_err(error::storage)?;
            return Ok(PasswordResetPreparation { delivery: None });
        }
        consume_open(&mut transaction, account_id).await?;
    }

    let code_digest = code_verification::password_reset_digest(
        input.verification_key,
        input.challenge_id,
        account_id,
        input.email,
        credential_version,
        input.code,
    );
    sqlx::query(
        "INSERT INTO password_reset_challenges \
         (id, account_id, email, credential_version, code_digest, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(input.challenge_id)
    .bind(account_id)
    .bind(input.email)
    .bind(credential_version)
    .bind(code_digest.as_slice())
    .bind(input.expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(PasswordResetPreparation {
        delivery: Some(PasswordResetDelivery {
            challenge_id: input.challenge_id,
            email: input.email.to_owned(),
        }),
    })
}

pub(crate) async fn begin_delivery(pool: &PgPool, challenge_id: Uuid) -> AppResult<bool> {
    let updated = sqlx::query(
        "UPDATE password_reset_challenges SET sent_at = now() \
         WHERE id = $1 AND sent_at IS NULL AND consumed_at IS NULL \
           AND expires_at > now()",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?
    .rows_affected();
    Ok(updated == 1)
}

pub(crate) async fn cancel_delivery(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE password_reset_challenges SET consumed_at = now() \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn snapshot(
    pool: &PgPool,
    normalized_email: &str,
) -> AppResult<Option<PasswordResetSnapshot>> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT challenge.id, challenge.account_id, challenge.email, \
                challenge.credential_version, challenge.code_digest, \
                challenge.attempt_count, challenge.expires_at, challenge.sent_at, \
                challenge.consumed_at, account.role, account.status, \
                account.email_verified_at, account.credential_version, \
                account.email, account.password_hash \
         FROM password_reset_challenges AS challenge \
         JOIN accounts AS account ON account.id = challenge.account_id \
         WHERE challenge.email = $1 AND challenge.consumed_at IS NULL \
         ORDER BY challenge.created_at DESC LIMIT 1",
    )
    .bind(normalized_email)
    .fetch_optional(pool)
    .await
    .map(|row| row.map(into_snapshot))
    .map_err(error::storage)
}

pub(crate) async fn reject_attempt(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    increment_or_consume(&mut transaction, challenge_id).await?;
    transaction.commit().await.map_err(error::storage)
}

pub(crate) async fn invalidate(pool: &PgPool, challenge_id: Uuid) -> AppResult<()> {
    sqlx::query(
        "UPDATE password_reset_challenges SET consumed_at = now() \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn complete(
    pool: &PgPool,
    snapshot: &PasswordResetSnapshot,
    supplied_digest: &[u8; 32],
    new_password_hash: &str,
) -> AppResult<bool> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let account = sqlx::query_as::<_, AccountRow>(
        "SELECT id, email, role, status, email_verified_at, credential_version, password_hash \
         FROM accounts WHERE id = $1 FOR UPDATE",
    )
    .bind(snapshot.account_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let challenge = lock_challenge(&mut transaction, snapshot.id).await?;
    let account_valid = account.is_some_and(
        |(
            account_id,
            email,
            role,
            status,
            email_verified_at,
            credential_version,
            password_hash,
        )| {
            account_id == snapshot.account_id
                && email.as_deref() == Some(snapshot.email.as_str())
                && email == snapshot.account_email
                && role == snapshot.role
                && matches!(role.as_str(), "user" | "admin")
                && status == "active"
                && email_verified_at.is_some()
                && credential_version == snapshot.credential_version
                && password_hash == snapshot.account_password_hash
        },
    );
    let challenge_valid = challenge.as_ref().is_some_and(|current| {
        current.id == snapshot.id
            && current.account_id == snapshot.account_id
            && current.email == snapshot.email
            && current.credential_version == snapshot.credential_version
            && current.sent_at.is_some()
            && current.consumed_at.is_none()
            && current.expires_at > Utc::now()
            && current.attempt_count < MAX_ATTEMPTS
            && code_verification::matches(&current.code_digest, supplied_digest)
    });
    if !account_valid || !challenge_valid {
        consume_one(&mut transaction, snapshot.id).await?;
        transaction.commit().await.map_err(error::storage)?;
        return Ok(false);
    }

    let new_version = snapshot
        .credential_version
        .checked_add(1)
        .ok_or_else(|| AppError::Internal("凭据版本已超出支持范围".to_owned()))?;
    let updated = sqlx::query(
        "UPDATE accounts SET password_hash = $2, credential_version = $3, \
             consecutive_login_failures = 0, login_locked_until = NULL, updated_at = now() \
         WHERE id = $1 AND role IN ('user', 'admin') AND status = 'active' \
           AND email_verified_at IS NOT NULL AND credential_version = $4 \
           AND email = $5 AND password_hash = $6",
    )
    .bind(snapshot.account_id)
    .bind(new_password_hash)
    .bind(new_version)
    .bind(snapshot.credential_version)
    .bind(&snapshot.email)
    .bind(&snapshot.account_password_hash)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?
    .rows_affected();
    if updated != 1 {
        transaction.rollback().await.map_err(error::storage)?;
        return Ok(false);
    }
    let sessions_revoked = sqlx::query(
        "UPDATE sessions SET revoked_at = now() \
         WHERE account_id = $1 AND revoked_at IS NULL",
    )
    .bind(snapshot.account_id)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?
    .rows_affected();
    consume_open(&mut transaction, snapshot.account_id).await?;
    sqlx::query(
        "UPDATE login_verification_challenges SET consumed_at = now() \
         WHERE account_id = $1 AND consumed_at IS NULL",
    )
    .bind(snapshot.account_id)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    audit_reset(&mut transaction, snapshot.account_id, sessions_revoked).await?;
    record_account_event(
        &mut transaction,
        snapshot.account_id,
        AccountNotificationEvent::PasswordChanged,
    )
    .await?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(true)
}

async fn lock_challenge(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<Option<PasswordResetSnapshot>> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT challenge.id, challenge.account_id, challenge.email, \
                challenge.credential_version, challenge.code_digest, \
                challenge.attempt_count, challenge.expires_at, challenge.sent_at, \
                challenge.consumed_at, account.role, account.status, \
                account.email_verified_at, account.credential_version, \
                account.email, account.password_hash \
         FROM password_reset_challenges AS challenge \
         JOIN accounts AS account ON account.id = challenge.account_id \
         WHERE challenge.id = $1 FOR UPDATE OF challenge",
    )
    .bind(challenge_id)
    .fetch_optional(&mut **transaction)
    .await
    .map(|row| row.map(into_snapshot))
    .map_err(error::storage)
}

async fn increment_or_consume(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE password_reset_challenges \
         SET attempt_count = LEAST(attempt_count + 1, $2), \
             consumed_at = CASE \
                 WHEN attempt_count + 1 >= $2 OR expires_at <= now() THEN now() \
                 ELSE consumed_at \
             END \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .bind(MAX_ATTEMPTS)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

async fn consume_open(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE password_reset_challenges SET consumed_at = now() \
         WHERE account_id = $1 AND consumed_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

async fn consume_one(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE password_reset_challenges SET consumed_at = now() \
         WHERE id = $1 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

async fn audit_reset(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    sessions_revoked: u64,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events \
         (id, actor_account_id, action, resource_kind, resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'account.password_reset', 'account', $3, 'success', $4, \
                 jsonb_build_object('sessions_revoked', $5::bigint))",
    )
    .bind(Uuid::now_v7())
    .bind(account_id)
    .bind(account_id.to_string())
    .bind(request_id)
    .bind(i64::try_from(sessions_revoked).unwrap_or(i64::MAX))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存密码重置审计失败".to_owned()))?;
    Ok(())
}

fn into_snapshot(value: SnapshotRow) -> PasswordResetSnapshot {
    PasswordResetSnapshot {
        id: value.0,
        account_id: value.1,
        email: value.2,
        credential_version: value.3,
        code_digest: value.4,
        attempt_count: value.5,
        expires_at: value.6,
        sent_at: value.7,
        consumed_at: value.8,
        role: value.9,
        status: value.10,
        email_verified_at: value.11,
        account_credential_version: value.12,
        account_email: value.13,
        account_password_hash: value.14,
    }
}

fn ineligible() -> PasswordResetPreparation {
    PasswordResetPreparation { delivery: None }
}
