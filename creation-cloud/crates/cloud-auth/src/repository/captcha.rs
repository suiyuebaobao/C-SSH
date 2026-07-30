//! 保存管理员登录 CAPTCHA 摘要并提供带尝试上限的一次性消费。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{captcha::MAX_ATTEMPTS, verification};

use super::error;

const MAX_OPEN_CHALLENGES: i64 = 4_096;
const ISSUE_LOCK_KEY: i64 = 0x4353_4341_5054_4348;

type ChallengeRow = (Vec<u8>, i32, DateTime<Utc>, Option<DateTime<Utc>>);

pub(crate) async fn insert(
    pool: &PgPool,
    challenge_id: Uuid,
    code_digest: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(ISSUE_LOCK_KEY)
        .execute(&mut *transaction)
        .await
        .map_err(error::storage)?;
    sqlx::query(
        "DELETE FROM admin_login_captcha_challenges \
         WHERE (consumed_at IS NOT NULL OR expires_at <= now()) \
           AND created_at < now() - interval '1 hour'",
    )
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let open = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM admin_login_captcha_challenges \
         WHERE consumed_at IS NULL AND expires_at > now()",
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if open >= MAX_OPEN_CHALLENGES {
        return Err(AppError::RateLimitedAfter {
            message: "图形验证码请求过于频繁，请稍后重试".to_owned(),
            retry_after_seconds: 60,
        });
    }
    sqlx::query(
        "INSERT INTO admin_login_captcha_challenges (id, code_digest, expires_at) \
         VALUES ($1, $2, $3)",
    )
    .bind(challenge_id)
    .bind(code_digest)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(())
}

pub(crate) async fn consume(
    transaction: &mut Transaction<'_, Postgres>,
    challenge_id: Uuid,
    supplied_digest: &[u8; 32],
) -> AppResult<bool> {
    let Some(challenge) = sqlx::query_as::<_, ChallengeRow>(
        "SELECT code_digest, attempt_count, expires_at, consumed_at \
         FROM admin_login_captcha_challenges WHERE id = $1 FOR UPDATE",
    )
    .bind(challenge_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    else {
        return Ok(false);
    };
    let valid = challenge.1 < MAX_ATTEMPTS
        && challenge.2 > Utc::now()
        && challenge.3.is_none()
        && verification::matches(&challenge.0, supplied_digest);
    if valid {
        sqlx::query(
            "UPDATE admin_login_captcha_challenges SET consumed_at = now() \
             WHERE id = $1 AND consumed_at IS NULL",
        )
        .bind(challenge_id)
        .execute(&mut **transaction)
        .await
        .map_err(error::storage)?;
        return Ok(true);
    }
    if challenge.3.is_none() {
        sqlx::query(
            "UPDATE admin_login_captcha_challenges \
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
    }
    Ok(false)
}
