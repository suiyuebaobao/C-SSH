//! Verifies a password-reset challenge and revokes every prior account session.

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;

use crate::{password, repository, validation, verification};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetPassword {
    pub email: String,
    pub code: String,
    pub new_password: String,
}

pub(crate) struct ResetPasswordOutcome {
    pub is_admin: bool,
}

impl ResetPassword {
    fn validate(&self) -> AppResult<()> {
        validation::password(&self.new_password)?;
        if self.code.len() != 6 || !self.code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid_code());
        }
        Ok(())
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    verification_key: &[u8],
    command: ResetPassword,
) -> AppResult<ResetPasswordOutcome> {
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable(
            "密码重置验证码密钥尚未安全配置".to_owned(),
        ));
    }
    command.validate()?;
    let email = validation::normalize_email(&command.email)?;
    let Some(snapshot) = repository::password_reset::snapshot(pool, &email).await? else {
        return Err(invalid_code());
    };
    let supplied_digest = verification::password_reset_digest(
        verification_key,
        snapshot.id,
        snapshot.account_id,
        &snapshot.email,
        snapshot.credential_version,
        &command.code,
    );
    let account_snapshot_valid = matches!(snapshot.role.as_str(), "user" | "admin")
        && snapshot.status == "active"
        && snapshot.email_verified_at.is_some()
        && snapshot.account_email.as_deref() == Some(snapshot.email.as_str())
        && snapshot.account_credential_version == snapshot.credential_version
        && !snapshot.account_password_hash.is_empty();
    let challenge_state_valid = snapshot.sent_at.is_some()
        && snapshot.consumed_at.is_none()
        && snapshot.expires_at > chrono::Utc::now()
        && snapshot.attempt_count < verification::MAX_ATTEMPTS;
    if !account_snapshot_valid || !challenge_state_valid {
        repository::password_reset::invalidate(pool, snapshot.id).await?;
        return Err(invalid_code());
    }
    if !verification::matches(&snapshot.code_digest, &supplied_digest) {
        repository::password_reset::reject_attempt(pool, snapshot.id).await?;
        return Err(invalid_code());
    }

    if password::verify(
        command.new_password.clone(),
        snapshot.account_password_hash.clone(),
    )
    .await?
    {
        return Err(AppError::Validation("新密码不能与当前密码相同".to_owned()));
    }
    let is_admin = snapshot.role == "admin";
    let new_password_hash = password::hash(command.new_password).await?;
    if !repository::password_reset::complete(pool, &snapshot, &supplied_digest, &new_password_hash)
        .await?
    {
        return Err(invalid_code());
    }
    Ok(ResetPasswordOutcome { is_admin })
}

fn invalid_code() -> AppError {
    AppError::Unauthorized("验证码无效、已过期或已达到尝试上限".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_requires_a_six_digit_code_and_a_policy_compliant_password() {
        assert!(
            ResetPassword {
                email: "user@example.com".to_owned(),
                code: "123456".to_owned(),
                new_password: "new-password-value".to_owned(),
            }
            .validate()
            .is_ok()
        );
        for code in ["12345", "1234567", "12a456"] {
            assert!(
                ResetPassword {
                    email: "user@example.com".to_owned(),
                    code: code.to_owned(),
                    new_password: "new-password-value".to_owned(),
                }
                .validate()
                .is_err()
            );
        }
    }
}
