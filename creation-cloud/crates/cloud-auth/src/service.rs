//! 持有认证域数据库连接并作为各认证用例的统一调用入口。

use std::time::Duration;

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{
    session::{AuthenticatedSession, IssuedSession},
    use_case::{self, ChangePassword, Login, Register},
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    session_ttl: Duration,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool, session_ttl: Duration) -> Self {
        Self { pool, session_ttl }
    }

    pub(crate) async fn register(&self, command: Register) -> AppResult<IssuedSession> {
        use_case::register::execute(&self.pool, self.session_ttl, command).await
    }

    pub(crate) async fn login(&self, command: Login) -> AppResult<IssuedSession> {
        use_case::login::execute(&self.pool, self.session_ttl, command).await
    }

    pub(crate) async fn logout(&self, session: &AuthenticatedSession) -> AppResult<()> {
        use_case::logout::execute(&self.pool, session).await
    }

    pub(crate) async fn change_password(
        &self,
        session: &AuthenticatedSession,
        command: ChangePassword,
    ) -> AppResult<()> {
        use_case::change_password::execute(&self.pool, session, command).await
    }

    /// 使用 Cookie 中的原始令牌完成会话鉴权。
    pub async fn authenticate(&self, raw_token: &str) -> AppResult<AuthenticatedSession> {
        use_case::session::authenticate(&self.pool, raw_token).await
    }
}
