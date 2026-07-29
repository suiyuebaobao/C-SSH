//! 创建待验证账号并在事务提交后投递一次性六位验证码。

use std::sync::Arc;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{mailer::VerificationMailer, password, repository, validation, verification};

#[derive(Deserialize)]
pub struct Register {
    pub email: String,
    pub password: String,
    pub display_name: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Serialize)]
pub struct RegistrationStatus {
    pub status: &'static str,
}

pub(crate) struct ValidatedRegister {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub locale: String,
}

impl Register {
    pub(crate) fn validate(self) -> AppResult<ValidatedRegister> {
        validation::password(&self.password)?;
        Ok(ValidatedRegister {
            email: validation::normalize_email(&self.email)?,
            password: self.password,
            display_name: validation::display_name(&self.display_name)?,
            locale: validation::locale(&self.locale)?,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    verification_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: Register,
) -> AppResult<RegistrationStatus> {
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable("邮箱验证密钥尚未安全配置".to_owned()));
    }
    let command = command.validate()?;
    let password_hash = password::hash(command.password).await?;
    let challenge_id = Uuid::now_v7();
    let code = verification::issue_code();
    let code_digest = verification::digest(verification_key, challenge_id, &command.email, &code);
    let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
    let should_send = repository::register::prepare(
        pool,
        repository::register::PendingAccount {
            account_id: Uuid::now_v7(),
            email: &command.email,
            password_hash: &password_hash,
            display_name: &command.display_name,
            locale: &command.locale,
            challenge_id,
            code_digest: &code_digest,
            expires_at,
        },
    )
    .await?;
    if should_send {
        dispatch(pool, mailer, challenge_id, &command.email, &code).await?;
    }
    Ok(RegistrationStatus {
        status: "verification_required",
    })
}

pub(crate) async fn dispatch(
    pool: &PgPool,
    mailer: &Arc<dyn VerificationMailer>,
    challenge_id: Uuid,
    email: &str,
    code: &str,
) -> AppResult<()> {
    if let Err(error) = mailer.send_verification(email, code).await {
        repository::verification::cancel_unsent(pool, challenge_id).await?;
        return Err(error);
    }
    repository::verification::mark_sent(pool, challenge_id).await
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}
