//! 持有认证域数据库连接并作为各认证用例的统一调用入口。

use std::time::Duration;

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::PgConnection;

use crate::{
    credential_limiter::CredentialLimiter,
    login_limiter::LoginLimiter,
    session::{AuthenticatedSession, IssuedSession},
    use_case::{self, ChangePassword, Login, Register},
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    session_ttl: Duration,
    credential_limiter: CredentialLimiter,
    login_limiter: LoginLimiter,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, session_ttl: Duration) -> Self {
        Self {
            pool,
            session_ttl,
            credential_limiter: CredentialLimiter::default(),
            login_limiter: LoginLimiter::default(),
        }
    }

    pub(crate) async fn register(&self, command: Register) -> AppResult<IssuedSession> {
        let _permit = self.credential_limiter.acquire_register(&command.email)?;
        use_case::register::execute(&self.pool, self.session_ttl, command).await
    }

    pub(crate) async fn login(&self, command: Login) -> AppResult<IssuedSession> {
        let _permit = self.login_limiter.acquire(&command.identifier)?;
        use_case::login::execute(&self.pool, self.session_ttl, command).await
    }

    pub(crate) async fn logout(&self, session: &AuthenticatedSession) -> AppResult<()> {
        use_case::logout::execute(&self.pool, session).await
    }

    pub async fn change_password(
        &self,
        session: &AuthenticatedSession,
        command: ChangePassword,
    ) -> AppResult<()> {
        let _permit = self
            .credential_limiter
            .acquire_password(session.account_id)?;
        use_case::change_password::execute(&self.pool, session, command).await
    }

    /// 使用 Cookie 中的原始令牌完成会话鉴权。
    pub async fn authenticate(&self, raw_token: &str) -> AppResult<AuthenticatedSession> {
        use_case::session::authenticate(&self.pool, raw_token).await
    }

    /// 删除截止时间以前已过期的一批会话，并返回数据库实际删除行数。
    pub async fn cleanup_expired_sessions(
        &self,
        delete_before: DateTime<Utc>,
        batch_size: u32,
    ) -> AppResult<u64> {
        use_case::cleanup_expired_sessions::execute(&self.pool, delete_before, batch_size).await
    }

    /// 在调用方持有的 PostgreSQL 会话上删除单批过期会话。
    pub async fn cleanup_expired_sessions_on_connection(
        &self,
        connection: &mut PgConnection,
        delete_before: DateTime<Utc>,
        batch_size: u32,
    ) -> AppResult<u64> {
        use_case::cleanup_expired_sessions::execute_on_connection(
            connection,
            delete_before,
            batch_size,
        )
        .await
    }
}
