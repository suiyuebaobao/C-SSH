//! 为仍处于 pending 状态的账号安全重发验证码。

use std::sync::Arc;

use chrono::Utc;
use cloud_domain::AppResult;
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{mailer::VerificationMailer, repository, validation, verification};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResendVerification {
    pub email: String,
}

#[derive(Serialize)]
pub struct ResendStatus {
    pub status: &'static str,
}

pub(crate) async fn execute(
    pool: &PgPool,
    verification_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: ResendVerification,
) -> AppResult<ResendStatus> {
    let email = validation::normalize_email(&command.email)?;
    let challenge_id = Uuid::now_v7();
    let code = verification::issue_code();
    let digest = verification::digest(verification_key, challenge_id, &email, &code);
    let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
    let should_send = repository::verification::prepare_resend(
        pool,
        verification_key.len() >= 32,
        &email,
        challenge_id,
        &digest,
        expires_at,
    )
    .await?;
    if should_send {
        super::register::dispatch(pool, mailer, challenge_id, &email, &code).await?;
    }
    Ok(ResendStatus {
        status: "verification_required",
    })
}
