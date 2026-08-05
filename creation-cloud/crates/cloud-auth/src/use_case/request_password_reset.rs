//! Accepts a non-enumerating password-reset request and dispatches mail after commit.

use std::sync::Arc;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    captcha::CaptchaPurpose, mailer::VerificationMailer, repository, validation, verification,
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestPasswordReset {
    pub email: String,
    #[serde(default)]
    pub captcha_id: Option<Uuid>,
    #[serde(default)]
    pub captcha_code: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct PasswordResetVerificationRequired {
    pub status: &'static str,
}

pub(crate) async fn execute(
    pool: &PgPool,
    verification_key: &[u8],
    captcha_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: RequestPasswordReset,
) -> AppResult<PasswordResetVerificationRequired> {
    let email = validation::normalize_email(&command.email)?;
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable(
            "密码重置验证码密钥尚未安全配置".to_owned(),
        ));
    }
    let captcha_id = command.captcha_id.unwrap_or_else(Uuid::nil);
    let captcha_code = command.captcha_code.as_deref().unwrap_or_default();
    let captcha_code_valid = valid_captcha_code(captcha_code);
    let captcha_digest = verification::captcha_digest(
        captcha_key,
        captcha_id,
        CaptchaPurpose::PasswordReset,
        if captcha_code_valid {
            captcha_code
        } else {
            "000000"
        },
    );
    let challenge_id = Uuid::now_v7();
    let code = verification::issue_code();
    let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
    let prepared = repository::password_reset::prepare(
        pool,
        repository::password_reset::PreparePasswordReset {
            email: &email,
            challenge_id,
            code: &code,
            verification_key,
            expires_at,
            captcha_id,
            captcha_digest: &captcha_digest,
            captcha_code_valid,
        },
    )
    .await?;
    if let Some(delivery) = prepared.delivery {
        dispatch_after_commit(
            pool.clone(),
            Arc::clone(mailer),
            delivery,
            code,
            sanitized_request_id(),
        );
    }
    Ok(PasswordResetVerificationRequired {
        status: "password_reset_request_accepted",
    })
}

fn dispatch_after_commit(
    pool: PgPool,
    mailer: Arc<dyn VerificationMailer>,
    delivery: repository::password_reset::PasswordResetDelivery,
    code: String,
    request_id: String,
) {
    tokio::spawn(async move {
        let delivery_started =
            match repository::password_reset::begin_delivery(&pool, delivery.challenge_id).await {
                Ok(started) => started,
                Err(_) => {
                    tracing::warn!(
                        event = "password_reset_mail_delivery",
                        stage = "claim",
                        result = "error",
                        request_id = %request_id,
                    );
                    return;
                }
            };
        if !delivery_started {
            tracing::info!(
                event = "password_reset_mail_delivery",
                stage = "claim",
                result = "not_claimed",
                request_id = %request_id,
            );
            return;
        }
        match mailer
            .send_password_reset(&delivery.email, &code, delivery.challenge_id)
            .await
        {
            Ok(()) => tracing::info!(
                event = "password_reset_mail_delivery",
                stage = "send",
                result = "success",
                request_id = %request_id,
            ),
            Err(_) => {
                tracing::warn!(
                    event = "password_reset_mail_delivery",
                    stage = "send",
                    result = "error",
                    request_id = %request_id,
                );
                match repository::password_reset::cancel_delivery(&pool, delivery.challenge_id)
                    .await
                {
                    Ok(()) => tracing::info!(
                        event = "password_reset_mail_delivery",
                        stage = "cancel",
                        result = "success",
                        request_id = %request_id,
                    ),
                    Err(_) => tracing::warn!(
                        event = "password_reset_mail_delivery",
                        stage = "cancel",
                        result = "error",
                        request_id = %request_id,
                    ),
                }
            }
        }
    });
}

fn sanitized_request_id() -> String {
    sanitize_request_id(cloud_domain::current_request_id())
}

fn sanitize_request_id(value: Option<String>) -> String {
    value
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn valid_captcha_code(code: &str) -> bool {
    code.len() == crate::captcha::CODE_LENGTH && code.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::sanitize_request_id;

    #[test]
    fn mail_delivery_logs_accept_only_a_bounded_ascii_request_id() {
        assert_eq!(
            sanitize_request_id(Some("019c1234-5678-7abc-8def-0123456789ab".to_owned())),
            "019c1234-5678-7abc-8def-0123456789ab"
        );
        for unsafe_value in ["", "request id", "id\nemail@example.test"] {
            assert_eq!(
                sanitize_request_id(Some(unsafe_value.to_owned())),
                "unavailable"
            );
        }
        assert_eq!(sanitize_request_id(None), "unavailable");
    }
}
