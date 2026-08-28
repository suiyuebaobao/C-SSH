//! Purpose-bound email recovery challenge for destructive Cloud-only reset.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Duration, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use rand::{Rng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    ProtectionResetChallengeResponse, VerifyProtectionResetChallengeRequest,
    VerifyProtectionResetChallengeResponse, actor::DeviceActor,
};

use super::super::{
    DbTransaction, SyncState, begin, commit, lock_sync_state, require_active_device, storage,
};

const CHALLENGE_TTL_MINUTES: i64 = 10;
const MAX_ATTEMPTS: i32 = 5;
const REQUEST_COOLDOWN_SECONDS: i64 = 60;
const CODE_CONTEXT: &[u8] = b"creation-cloud-protection-reset-code-v1\0";
const AUTH_CONTEXT: &[u8] = b"creation-cloud-protection-reset-authorization-v1\0";

pub(crate) struct PendingResetChallenge {
    pub(crate) response: ProtectionResetChallengeResponse,
    pub(crate) email: String,
    pub(crate) code: SecretCode,
}

pub(crate) struct SecretCode(Vec<u8>);

impl SecretCode {
    pub(crate) fn expose(&self) -> &str {
        std::str::from_utf8(&self.0).expect("issued verification code is ASCII")
    }
}

impl Drop for SecretCode {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

#[derive(FromRow)]
struct ChallengeRow {
    account_id: Uuid,
    device_id: Uuid,
    email: String,
    credential_version: i64,
    sync_generation: i64,
    protection_epoch: i64,
    protection_revision: i64,
    current_revision: i64,
    code_digest: Vec<u8>,
    attempt_count: i32,
    authorization_digest: Option<Vec<u8>>,
    expires_at: DateTime<Utc>,
    sent_at: Option<DateTime<Utc>>,
    verified_at: Option<DateTime<Utc>>,
    consumed_at: Option<DateTime<Utc>>,
}

pub(crate) async fn issue_reset_challenge(
    pool: &PgPool,
    actor: DeviceActor,
    verification_key: &[u8],
) -> AppResult<PendingResetChallenge> {
    require_key(verification_key)?;
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    let (email, credential_version) = sqlx::query_as::<_, (String, i64)>(
        "SELECT email, credential_version FROM accounts
         WHERE id=$1 AND status='active' AND email IS NOT NULL
           AND email_verified_at IS NOT NULL FOR SHARE",
    )
    .bind(actor.account_id())
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or_else(|| AppError::Conflict("active verified email is required".to_owned()))?;
    if let Some(created_at) = sqlx::query_scalar::<_, DateTime<Utc>>(
        "SELECT created_at FROM cloud_data_protection_reset_challenges
         WHERE account_id=$1 ORDER BY created_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(actor.account_id())
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
        && created_at + Duration::seconds(REQUEST_COOLDOWN_SECONDS) > Utc::now()
    {
        return Err(AppError::RateLimitedAfter {
            message: "数据保护清空验证码请求过于频繁".to_owned(),
            retry_after_seconds: u64::try_from(REQUEST_COOLDOWN_SECONDS).unwrap_or(60),
        });
    }
    sqlx::query(
        "UPDATE cloud_data_protection_reset_challenges
         SET consumed_at=now()
         WHERE account_id=$1 AND consumed_at IS NULL",
    )
    .bind(actor.account_id())
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    let challenge_id = Uuid::now_v7();
    let code = format!("{:06}", rand::rng().random_range(0_u32..1_000_000));
    let code_digest = code_digest(
        verification_key,
        challenge_id,
        actor.account_id(),
        actor.device_id(),
        credential_version,
        state,
        &email,
        &code,
    );
    let expires_at = Utc::now() + Duration::minutes(CHALLENGE_TTL_MINUTES);
    sqlx::query(
        "INSERT INTO cloud_data_protection_reset_challenges
             (id,account_id,device_id,email,credential_version,sync_generation,
              protection_epoch,protection_revision,current_revision,code_digest,expires_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
    )
    .bind(challenge_id)
    .bind(actor.account_id())
    .bind(actor.device_id())
    .bind(&email)
    .bind(credential_version)
    .bind(state.sync_generation)
    .bind(state.protection_epoch)
    .bind(state.protection_revision)
    .bind(state.current_revision)
    .bind(code_digest.as_slice())
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    commit(tx).await?;
    Ok(PendingResetChallenge {
        response: ProtectionResetChallengeResponse {
            status: "verification_required".to_owned(),
            challenge_id,
            expires_at,
        },
        email,
        code: SecretCode(code.into_bytes()),
    })
}

pub(crate) async fn mark_challenge_sent(
    pool: &PgPool,
    actor: DeviceActor,
    challenge_id: Uuid,
) -> AppResult<()> {
    let updated = sqlx::query(
        "UPDATE cloud_data_protection_reset_challenges
         SET sent_at=now()
         WHERE id=$1 AND account_id=$2 AND device_id=$3
           AND consumed_at IS NULL AND sent_at IS NULL AND expires_at>now()",
    )
    .bind(challenge_id)
    .bind(actor.account_id())
    .bind(actor.device_id())
    .execute(pool)
    .await
    .map_err(storage)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(AppError::Conflict(
            "protection reset challenge is no longer active".to_owned(),
        ))
    }
}

pub(crate) async fn cancel_challenge(
    pool: &PgPool,
    actor: DeviceActor,
    challenge_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE cloud_data_protection_reset_challenges SET consumed_at=now()
         WHERE id=$1 AND account_id=$2 AND device_id=$3 AND consumed_at IS NULL",
    )
    .bind(challenge_id)
    .bind(actor.account_id())
    .bind(actor.device_id())
    .execute(pool)
    .await
    .map_err(storage)?;
    Ok(())
}

pub(crate) async fn verify_reset_challenge(
    pool: &PgPool,
    actor: DeviceActor,
    request: &VerifyProtectionResetChallengeRequest,
    verification_key: &[u8],
) -> AppResult<VerifyProtectionResetChallengeResponse> {
    require_key(verification_key)?;
    let mut tx = begin(pool).await?;
    require_active_device(&mut tx, actor.account_id(), actor.device_id()).await?;
    let state = lock_sync_state(&mut tx, actor.account_id()).await?;
    let row = load_challenge(&mut tx, request.challenge_id).await?;
    if !challenge_is_usable(&row, actor, false)
        || !matches_current_account(&mut tx, &row).await?
        || !challenge_matches_state(&row, state)
    {
        commit(tx).await?;
        return Err(invalid_challenge());
    }
    let actual = code_digest(
        verification_key,
        request.challenge_id,
        actor.account_id(),
        actor.device_id(),
        row.credential_version,
        state,
        &row.email,
        &request.code,
    );
    if !constant_time_eq(&row.code_digest, &actual) {
        let next_attempt = row.attempt_count.saturating_add(1).min(MAX_ATTEMPTS);
        sqlx::query(
            "UPDATE cloud_data_protection_reset_challenges
             SET attempt_count=$2,
                 consumed_at=CASE WHEN $2 >= 5 THEN now() ELSE consumed_at END
             WHERE id=$1",
        )
        .bind(request.challenge_id)
        .bind(next_attempt)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        commit(tx).await?;
        return Err(invalid_challenge());
    }
    let mut token = [0_u8; 32];
    rand::rng().fill_bytes(&mut token);
    let digest = authorization_digest(
        verification_key,
        request.challenge_id,
        actor.account_id(),
        actor.device_id(),
        state,
        &token,
    );
    sqlx::query(
        "UPDATE cloud_data_protection_reset_challenges
         SET authorization_digest=$2, verified_at=now()
         WHERE id=$1 AND verified_at IS NULL AND consumed_at IS NULL",
    )
    .bind(request.challenge_id)
    .bind(digest.as_slice())
    .execute(&mut *tx)
    .await
    .map_err(storage)?;
    commit(tx).await?;
    let authorization_token = STANDARD.encode(token);
    token.fill(0);
    Ok(VerifyProtectionResetChallengeResponse {
        status: "verified".to_owned(),
        challenge_id: request.challenge_id,
        authorization_token,
        expires_at: row.expires_at,
    })
}

pub(crate) async fn consume_email_authorization(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    challenge_id: Uuid,
    state: SyncState,
    token: &[u8],
    verification_key: &[u8],
) -> AppResult<()> {
    require_key(verification_key)?;
    if token.len() != 32 {
        return Err(invalid_challenge());
    }
    let row = load_challenge(tx, challenge_id).await?;
    if !challenge_is_usable(&row, actor, true)
        || !matches_current_account(tx, &row).await?
        || !challenge_matches_state(&row, state)
    {
        return Err(invalid_challenge());
    }
    let expected = row.authorization_digest.ok_or_else(invalid_challenge)?;
    let actual = authorization_digest(
        verification_key,
        challenge_id,
        actor.account_id(),
        actor.device_id(),
        state,
        token,
    );
    if !constant_time_eq(&expected, &actual) {
        return Err(invalid_challenge());
    }
    let updated = sqlx::query(
        "UPDATE cloud_data_protection_reset_challenges SET consumed_at=now()
         WHERE id=$1 AND consumed_at IS NULL AND expires_at>now()",
    )
    .bind(challenge_id)
    .execute(&mut **tx)
    .await
    .map_err(storage)?
    .rows_affected();
    if updated == 1 {
        Ok(())
    } else {
        Err(invalid_challenge())
    }
}

async fn load_challenge(tx: &mut DbTransaction<'_>, challenge_id: Uuid) -> AppResult<ChallengeRow> {
    sqlx::query_as(
        "SELECT account_id,device_id,email,credential_version,sync_generation,
                protection_epoch,protection_revision,current_revision,code_digest,
                attempt_count,authorization_digest,expires_at,sent_at,
                verified_at,consumed_at
         FROM cloud_data_protection_reset_challenges WHERE id=$1 FOR UPDATE",
    )
    .bind(challenge_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?
    .ok_or_else(invalid_challenge)
}

fn challenge_matches_state(row: &ChallengeRow, state: SyncState) -> bool {
    row.sync_generation == state.sync_generation
        && row.protection_epoch == state.protection_epoch
        && row.protection_revision == state.protection_revision
        && row.current_revision == state.current_revision
}

fn challenge_is_usable(row: &ChallengeRow, actor: DeviceActor, require_verified: bool) -> bool {
    if row.account_id != actor.account_id()
        || row.device_id != actor.device_id()
        || row.sent_at.is_none()
        || row.consumed_at.is_some()
        || row.expires_at <= Utc::now()
        || row.attempt_count >= MAX_ATTEMPTS
        || (require_verified && row.verified_at.is_none())
        || (!require_verified && row.verified_at.is_some())
    {
        return false;
    }
    true
}

async fn matches_current_account(
    tx: &mut DbTransaction<'_>,
    row: &ChallengeRow,
) -> AppResult<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS(
             SELECT 1 FROM accounts
             WHERE id=$1 AND status='active' AND email=$2
               AND email_verified_at IS NOT NULL AND credential_version=$3
         )",
    )
    .bind(row.account_id)
    .bind(&row.email)
    .bind(row.credential_version)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)
}

#[allow(clippy::too_many_arguments)]
fn code_digest(
    key: &[u8],
    challenge_id: Uuid,
    account_id: Uuid,
    device_id: Uuid,
    credential_version: i64,
    state: SyncState,
    email: &str,
    code: &str,
) -> [u8; 32] {
    let mut message = Vec::with_capacity(CODE_CONTEXT.len() + email.len() + code.len() + 64);
    message.extend_from_slice(CODE_CONTEXT);
    message.extend_from_slice(challenge_id.as_bytes());
    message.extend_from_slice(account_id.as_bytes());
    message.extend_from_slice(device_id.as_bytes());
    message.extend_from_slice(&credential_version.to_be_bytes());
    append_state(&mut message, state);
    message.extend_from_slice(email.as_bytes());
    message.push(0);
    message.extend_from_slice(code.as_bytes());
    let result = hmac_sha256(key, &message);
    message.fill(0);
    result
}

fn authorization_digest(
    key: &[u8],
    challenge_id: Uuid,
    account_id: Uuid,
    device_id: Uuid,
    state: SyncState,
    token: &[u8],
) -> [u8; 32] {
    let mut message = Vec::with_capacity(AUTH_CONTEXT.len() + token.len() + 48);
    message.extend_from_slice(AUTH_CONTEXT);
    message.extend_from_slice(challenge_id.as_bytes());
    message.extend_from_slice(account_id.as_bytes());
    message.extend_from_slice(device_id.as_bytes());
    append_state(&mut message, state);
    message.extend_from_slice(token);
    let result = hmac_sha256(key, &message);
    message.fill(0);
    result
}

fn append_state(message: &mut Vec<u8>, state: SyncState) {
    message.extend_from_slice(&state.sync_generation.to_be_bytes());
    message.extend_from_slice(&state.protection_epoch.to_be_bytes());
    message.extend_from_slice(&state.protection_revision.to_be_bytes());
    message.extend_from_slice(&state.current_revision.to_be_bytes());
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut normalized = [0_u8; BLOCK];
    if key.len() > BLOCK {
        normalized[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner = [0x36_u8; BLOCK];
    let mut outer = [0x5c_u8; BLOCK];
    for index in 0..BLOCK {
        inner[index] ^= normalized[index];
        outer[index] ^= normalized[index];
    }
    let mut digest = Sha256::new();
    digest.update(inner);
    digest.update(message);
    let mut inside: [u8; 32] = digest.finalize().into();
    let mut digest = Sha256::new();
    digest.update(outer);
    digest.update(inside);
    let result = digest.finalize().into();
    normalized.fill(0);
    inner.fill(0);
    outer.fill(0);
    inside.fill(0);
    result
}

fn constant_time_eq(expected: &[u8], actual: &[u8; 32]) -> bool {
    if expected.len() != actual.len() {
        return false;
    }
    expected
        .iter()
        .zip(actual)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

fn require_key(key: &[u8]) -> AppResult<()> {
    if key.len() >= 32 {
        Ok(())
    } else {
        Err(AppError::Unavailable(
            "数据保护清空邮箱验证服务尚未配置".to_owned(),
        ))
    }
}

fn invalid_challenge() -> AppError {
    AppError::Validation("数据保护清空验证码或授权无效".to_owned())
}

include!("challenge/tests.rs");
