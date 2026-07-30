//! 校验当前密码并强制撤销所有旧凭据版本会话。

use std::time::Duration;

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    password, repository,
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    token, validation,
};

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
    session_ttl: Duration,
    session: &AuthenticatedSession,
    command: ChangePassword,
) -> AppResult<IssuedSession> {
    command.validate()?;
    let _always_revoke_all_sessions = command.revoke_other_sessions;
    let snapshot =
        repository::change_password::current_snapshot(pool, session.account_id, session.session_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("账号或当前会话不可用".to_owned()))?;
    if !password::verify(command.current_password, snapshot.password_hash.clone()).await? {
        return Err(AppError::Unauthorized("当前密码错误".to_owned()));
    }
    let new_hash = password::hash(command.new_password).await?;
    let now = Utc::now();
    let (session_kind, device_id, idle_expires_at, absolute_expires_at) =
        if snapshot.session_kind == "device" {
            let absolute = snapshot.absolute_expires_at;
            (
                "device",
                snapshot.device_id,
                std::cmp::min(now + chrono::Duration::days(90), absolute),
                absolute,
            )
        } else {
            let expiry = now
                + chrono::Duration::from_std(session_ttl)
                    .map_err(|_| AppError::Internal("会话有效期配置超出支持范围".to_owned()))?;
            ("unbound", None, expiry, expiry)
        };
    let session_id = Uuid::now_v7();
    let (raw_token, token_hash) = token::issue();
    repository::change_password::update_and_rotate(
        pool,
        session.account_id,
        session.session_id,
        &snapshot.password_hash,
        snapshot.credential_version,
        &new_hash,
        session_id,
        &token_hash,
        session_kind,
        device_id,
        idle_expires_at,
        absolute_expires_at,
    )
    .await?;
    Ok(IssuedSession {
        session: AuthenticatedSession {
            session_id,
            account_id: session.account_id,
            email: session.email.clone(),
            admin_login_name: session.admin_login_name.clone(),
            role: session.role.clone(),
            device_id,
            expires_at: idle_expires_at,
            csrf_token: token::csrf(&raw_token),
        },
        metadata: SessionMetadata {
            email_verified: !session.email.is_empty(),
            session_kind: session_kind.to_owned(),
            idle_expires_at,
            absolute_expires_at,
        },
        raw_token,
    })
}

const fn default_revoke_other_sessions() -> bool {
    true
}
