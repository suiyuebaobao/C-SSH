//! 以模糊错误校验账号密码并创建新的安全会话。

use std::time::Duration;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    password, repository,
    session::{AuthenticatedSession, IssuedSession},
    token, validation,
};

const INVALID_CREDENTIALS: &str = "邮箱或密码错误";

#[derive(Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl Login {
    pub(crate) fn validate(self) -> AppResult<Self> {
        Ok(Self {
            email: validation::normalize_email(&self.email)?,
            password: self.password,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session_ttl: Duration,
    command: Login,
) -> AppResult<IssuedSession> {
    let command = command.validate()?;
    let Some(account) = repository::login::find_account(pool, &command.email).await? else {
        // 未命中账号时仍执行一次 Argon2id，降低账号枚举的时序差异。
        let _ = password::hash(command.password).await?;
        return Err(invalid_credentials());
    };
    let password_valid = password::verify(command.password, account.password_hash).await?;
    if !password_valid || account.status != "active" {
        return Err(invalid_credentials());
    }

    let session_id = Uuid::now_v7();
    let expires_at = Utc::now()
        + chrono::Duration::from_std(session_ttl)
            .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?;
    let (raw_token, token_hash) = token::issue();
    repository::login::insert_session(pool, session_id, account.id, &token_hash, expires_at)
        .await?;

    Ok(IssuedSession {
        session: AuthenticatedSession {
            session_id,
            account_id: account.id,
            email: account.email,
            role: account.role,
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        raw_token,
    })
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized(INVALID_CREDENTIALS.to_owned())
}
