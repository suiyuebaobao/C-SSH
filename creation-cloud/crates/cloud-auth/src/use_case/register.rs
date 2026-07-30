//! 创建待验证账号并在事务提交后投递一次性六位验证码。

use std::sync::Arc;

use chrono::Utc;
use cloud_domain::AppResult;
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    mailer::{VerificationMailer, VerificationPurpose},
    password, repository,
    repository::register::RegistrationPreparation,
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    token, validation, verification,
};

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

pub enum RegistrationOutcome {
    VerificationRequired(RegistrationStatus),
    Session(Box<IssuedSession>),
    Accepted(RegistrationStatus),
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
    session_ttl: std::time::Duration,
    verification_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: Register,
) -> AppResult<RegistrationOutcome> {
    let command = command.validate()?;
    let password_hash = password::hash(command.password).await?;
    let challenge_id = Uuid::now_v7();
    let code = verification::issue_code();
    let code_digest = verification::digest(verification_key, challenge_id, &command.email, &code);
    let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
    let session_expires_at = Utc::now()
        + chrono::Duration::from_std(session_ttl).map_err(|_| {
            cloud_domain::AppError::Internal("会话有效期配置超出支持范围".to_owned())
        })?;
    let session_id = Uuid::now_v7();
    let (raw_token, token_hash) = token::issue();
    let preparation = repository::register::prepare(
        pool,
        verification_key.len() >= 32,
        repository::register::RegistrationAccount {
            account_id: Uuid::now_v7(),
            email: &command.email,
            password_hash: &password_hash,
            display_name: &command.display_name,
            locale: &command.locale,
            challenge_id,
            code_digest: &code_digest,
            expires_at,
            session_id,
            session_token_hash: &token_hash,
            session_expires_at,
        },
    )
    .await?;
    match preparation {
        RegistrationPreparation::Verification { should_send } => {
            if should_send {
                dispatch(pool, mailer, challenge_id, &command.email, &code).await?;
            }
            Ok(RegistrationOutcome::VerificationRequired(
                RegistrationStatus {
                    status: "verification_required",
                },
            ))
        }
        RegistrationPreparation::Session { account_id } => {
            Ok(RegistrationOutcome::Session(Box::new(IssuedSession {
                raw_token: raw_token.clone(),
                session: AuthenticatedSession {
                    session_id,
                    account_id,
                    email: command.email,
                    admin_login_name: None,
                    role: "user".to_owned(),
                    device_id: None,
                    expires_at: session_expires_at,
                    csrf_token: token::csrf(&raw_token),
                },
                metadata: SessionMetadata::unbound(session_expires_at, true),
            })))
        }
        RegistrationPreparation::Accepted => {
            Ok(RegistrationOutcome::Accepted(RegistrationStatus {
                status: "registration_accepted",
            }))
        }
    }
}

pub(crate) async fn dispatch(
    pool: &PgPool,
    mailer: &Arc<dyn VerificationMailer>,
    challenge_id: Uuid,
    email: &str,
    code: &str,
) -> AppResult<()> {
    if let Err(error) = mailer
        .send_verification(email, code, VerificationPurpose::Registration)
        .await
    {
        repository::verification::cancel_unsent(pool, challenge_id).await?;
        return Err(error);
    }
    repository::verification::mark_sent(pool, challenge_id).await
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}
