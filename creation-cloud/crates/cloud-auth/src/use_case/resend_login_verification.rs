//! 使用既有不透明挑战标识重发普通用户或管理员的登录邮箱验证码。

use std::sync::Arc;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    mailer::{VerificationMailer, VerificationPurpose},
    repository, verification,
};

use super::login::LoginVerificationRequired;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResendLoginVerification {
    pub challenge_id: Uuid,
}

pub(crate) async fn execute(
    pool: &PgPool,
    verification_key: &[u8],
    mailer: &Arc<dyn VerificationMailer>,
    command: ResendLoginVerification,
) -> AppResult<LoginVerificationRequired> {
    let account_id = repository::login_verification::find_account_id(pool, command.challenge_id)
        .await?
        .ok_or_else(invalid_challenge)?;
    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let account = repository::login::lock_by_id(&mut transaction, account_id)
        .await?
        .ok_or_else(invalid_challenge)?;
    super::login_lockout::ensure_not_locked(&account, Utc::now())?;
    let auth_settings = repository::settings::lock(&mut transaction).await?;
    let verification_enabled = super::login::requires_login_verification(
        &account,
        auth_settings.email_verification_enabled,
        auth_settings.admin_email_verification_enabled,
    );
    if !verification_enabled {
        transaction
            .commit()
            .await
            .map_err(repository::error::storage)?;
        return Err(invalid_challenge());
    }
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable(
            "登录验证码密钥尚未安全配置".to_owned(),
        ));
    }
    let current =
        repository::login_verification::lock_by_id(&mut transaction, command.challenge_id)
            .await?
            .filter(|value| value.account_id == account.id)
            .ok_or_else(invalid_challenge)?;
    let snapshot_valid = matches!(account.role.as_str(), "user" | "admin")
        && account.status == "active"
        && account.email_verified_at.is_some()
        && account.email.as_deref() == Some(current.email.as_str())
        && account.credential_version == current.credential_version
        && current.sent_at.is_some()
        && current.consumed_at.is_none()
        && current.attempt_count < verification::MAX_ATTEMPTS;
    if !snapshot_valid {
        return Err(invalid_challenge());
    }

    let challenge_id = Uuid::now_v7();
    let code = verification::issue_code();
    let expires_at = Utc::now() + chrono::Duration::minutes(verification::CODE_TTL_MINUTES);
    let code_digest = verification::login_digest(
        verification_key,
        challenge_id,
        account.id,
        &current.email,
        account.credential_version,
        &code,
    );
    repository::login_verification::replace_open(
        &mut transaction,
        repository::login_verification::NewLoginChallenge {
            id: challenge_id,
            account_id: account.id,
            email: &current.email,
            credential_version: account.credential_version,
            code_digest: &code_digest,
            expires_at,
        },
        auth_settings.email_cooldown_seconds,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;
    if let Err(error) = mailer
        .send_verification(&current.email, &code, VerificationPurpose::Login)
        .await
    {
        repository::login_verification::cancel_unsent(pool, challenge_id).await?;
        return Err(error);
    }
    repository::login_verification::mark_sent(pool, challenge_id).await?;
    Ok(LoginVerificationRequired {
        status: "verification_required",
        challenge_id,
        expires_at,
    })
}

fn invalid_challenge() -> AppError {
    AppError::Unauthorized("登录验证码无效、已过期或已达到尝试上限".to_owned())
}
