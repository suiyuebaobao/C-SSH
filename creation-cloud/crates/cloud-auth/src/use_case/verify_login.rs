//! 校验普通用户的登录邮箱验证码并在同一事务中签发会话。

use std::time::Duration;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    repository,
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    token, verification,
};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyLogin {
    pub challenge_id: Uuid,
    pub code: String,
}

impl VerifyLogin {
    fn validate(self) -> AppResult<Self> {
        if self.code.len() != 6 || !self.code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid_code());
        }
        Ok(self)
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session_ttl: Duration,
    verification_key: &[u8],
    command: VerifyLogin,
) -> AppResult<IssuedSession> {
    if verification_key.len() < 32 {
        return Err(AppError::Unavailable(
            "登录验证码密钥尚未安全配置".to_owned(),
        ));
    }
    let command = command.validate()?;
    let account_id = repository::login_verification::find_account_id(pool, command.challenge_id)
        .await?
        .ok_or_else(invalid_code)?;
    let session_id = Uuid::now_v7();
    let expires_at = super::login::session_expiry(session_ttl)?;
    let (raw_token, token_hash) = token::issue();
    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let account = repository::login::lock_by_id(&mut transaction, account_id)
        .await?
        .ok_or_else(invalid_code)?;
    let challenge =
        repository::login_verification::lock_by_id(&mut transaction, command.challenge_id)
            .await?
            .filter(|value| value.account_id == account.id)
            .ok_or_else(invalid_code)?;
    let supplied_digest = verification::login_digest(
        verification_key,
        challenge.id,
        challenge.account_id,
        &challenge.email,
        challenge.credential_version,
        &command.code,
    );
    let snapshot_valid = account.role == "user"
        && account.status == "active"
        && account.email_verified_at.is_some()
        && account.email.as_deref() == Some(challenge.email.as_str())
        && account.credential_version == challenge.credential_version
        && challenge.sent_at.is_some()
        && challenge.consumed_at.is_none()
        && challenge.expires_at > Utc::now()
        && challenge.attempt_count < verification::MAX_ATTEMPTS;
    let code_valid =
        snapshot_valid && verification::matches(&challenge.code_digest, &supplied_digest);
    if !code_valid {
        if snapshot_valid {
            repository::login_verification::increment_attempt(&mut transaction, challenge.id)
                .await?;
        } else if challenge.consumed_at.is_none() {
            repository::login_verification::consume(&mut transaction, challenge.id).await?;
        }
        transaction
            .commit()
            .await
            .map_err(repository::error::storage)?;
        return Err(invalid_code());
    }

    repository::login_verification::consume(&mut transaction, challenge.id).await?;
    repository::login::insert_session(
        &mut transaction,
        session_id,
        account.id,
        &token_hash,
        expires_at,
        account.credential_version,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;
    Ok(IssuedSession {
        raw_token: raw_token.clone(),
        session: AuthenticatedSession {
            session_id,
            account_id: account.id,
            email: account.email.unwrap_or_default(),
            admin_login_name: None,
            role: account.role,
            device_id: None,
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        metadata: SessionMetadata::unbound(expires_at, true),
    })
}

fn invalid_code() -> AppError {
    AppError::Unauthorized("登录验证码无效、已过期或已达到尝试上限".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_code_is_exactly_six_ascii_digits() {
        let challenge_id = Uuid::now_v7();
        assert!(
            VerifyLogin {
                challenge_id,
                code: "123456".to_owned(),
            }
            .validate()
            .is_ok()
        );
        for code in ["12345", "1234567", "１２３４５６", "12a456"] {
            assert!(
                VerifyLogin {
                    challenge_id,
                    code: code.to_owned(),
                }
                .validate()
                .is_err(),
                "{code}"
            );
        }
    }
}
