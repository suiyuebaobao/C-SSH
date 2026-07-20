//! 校验注册资料、生成密码哈希并原子创建账号、资料和首个会话。

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

#[derive(Deserialize)]
pub struct Register {
    pub email: String,
    pub password: String,
    pub display_name: String,
    #[serde(default = "default_locale")]
    pub locale: String,
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
    session_ttl: Duration,
    command: Register,
) -> AppResult<IssuedSession> {
    let command = command.validate()?;
    let password_hash = password::hash(command.password).await?;
    let account_id = Uuid::now_v7();
    let session_id = Uuid::now_v7();
    let expires_at = Utc::now()
        + chrono::Duration::from_std(session_ttl)
            .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?;
    let (raw_token, token_hash) = token::issue();

    repository::register::insert(
        pool,
        repository::register::NewAccount {
            account_id,
            email: &command.email,
            password_hash: &password_hash,
            display_name: &command.display_name,
            locale: &command.locale,
            session_id,
            token_hash: &token_hash,
            expires_at,
        },
    )
    .await?;

    Ok(IssuedSession {
        session: AuthenticatedSession {
            session_id,
            account_id,
            email: command.email,
            role: "user".to_owned(),
            expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        raw_token,
    })
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}
