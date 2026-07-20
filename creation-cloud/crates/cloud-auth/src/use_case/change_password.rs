//! 校验当前密码、更新 Argon2id 哈希，并按请求撤销其他会话。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;

use crate::{password, repository, session::AuthenticatedSession, validation};

#[derive(Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
    #[serde(default = "default_revoke_other_sessions")]
    pub revoke_other_sessions: bool,
}

impl ChangePassword {
    pub(crate) fn validate(&self) -> AppResult<()> {
        validation::password(&self.new_password)?;
        if self.current_password == self.new_password {
            return Err(AppError::Validation("新密码不能与当前密码相同".to_owned()));
        }
        Ok(())
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    command: ChangePassword,
) -> AppResult<()> {
    command.validate()?;
    let password_hash = repository::change_password::current_hash(pool, session.account_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("账号不可用".to_owned()))?;
    if !password::verify(command.current_password, password_hash).await? {
        return Err(AppError::Unauthorized("当前密码错误".to_owned()));
    }
    let new_hash = password::hash(command.new_password).await?;
    repository::change_password::update(
        pool,
        session.account_id,
        session.session_id,
        &new_hash,
        command.revoke_other_sessions,
    )
    .await
}

const fn default_revoke_other_sessions() -> bool {
    true
}
