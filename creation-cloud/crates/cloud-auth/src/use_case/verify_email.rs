//! 校验一次性邮箱验证码，激活普通账号并签发短期未绑定会话。

use std::time::Duration;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{repository, session::IssuedSession, token, validation, verification};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyEmail {
    pub email: String,
    pub code: String,
}

impl VerifyEmail {
    fn validate(self) -> AppResult<Self> {
        let email = validation::normalize_email(&self.email)?;
        if self.code.len() != 6 || !self.code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid_code());
        }
        Ok(Self {
            email,
            code: self.code,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session_ttl: Duration,
    verification_key: &[u8],
    request_metadata: &crate::TrustedRequestMetadata,
    command: VerifyEmail,
) -> AppResult<IssuedSession> {
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable("邮箱验证密钥尚未安全配置".to_owned()));
    }
    let command = command.validate()?;
    let expires_at = Utc::now()
        + chrono::Duration::from_std(session_ttl)
            .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?;
    let session_id = Uuid::now_v7();
    let (raw_token, token_hash) = token::issue();
    repository::verification::verify_and_issue(
        pool,
        &command.email,
        |challenge_id| {
            verification::digest(
                verification_key,
                challenge_id,
                &command.email,
                &command.code,
            )
        },
        expires_at,
        session_id,
        raw_token,
        token_hash,
        request_metadata,
    )
    .await
}

fn invalid_code() -> AppError {
    AppError::Unauthorized("验证码无效、已过期或已达到尝试上限".to_owned())
}
