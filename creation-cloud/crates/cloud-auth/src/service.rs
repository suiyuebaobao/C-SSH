//! 持有认证域数据库连接并作为各认证用例的统一调用入口。

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::PgConnection;

use crate::{
    credential_limiter::CredentialLimiter,
    login_limiter::LoginLimiter,
    mailer::{UnavailableVerificationMailer, VerificationMailer},
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    use_case::{
        self, ChangePassword, Login, Register, RegistrationStatus, ResendStatus,
        ResendVerification, VerifyEmail,
    },
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    session_ttl: Duration,
    verification_key: Arc<[u8]>,
    verification_mailer: Arc<dyn VerificationMailer>,
    credential_limiter: CredentialLimiter,
    login_limiter: LoginLimiter,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, session_ttl: Duration) -> Self {
        Self {
            pool,
            session_ttl,
            verification_key: Vec::<u8>::new().into(),
            verification_mailer: Arc::new(UnavailableVerificationMailer),
            credential_limiter: CredentialLimiter::default(),
            login_limiter: LoginLimiter::default(),
        }
    }

    #[must_use]
    pub fn with_verification(
        pool: PgPool,
        session_ttl: Duration,
        verification_key: Vec<u8>,
        verification_mailer: Arc<dyn VerificationMailer>,
    ) -> Self {
        Self {
            pool,
            session_ttl,
            verification_key: Arc::from(verification_key),
            verification_mailer,
            credential_limiter: CredentialLimiter::default(),
            login_limiter: LoginLimiter::default(),
        }
    }

    pub(crate) async fn register(&self, command: Register) -> AppResult<RegistrationStatus> {
        let _permit = self.credential_limiter.acquire_register(&command.email)?;
        use_case::register::execute(
            &self.pool,
            &self.verification_key,
            &self.verification_mailer,
            command,
        )
        .await
    }

    pub(crate) async fn resend_verification(
        &self,
        command: ResendVerification,
    ) -> AppResult<ResendStatus> {
        let _permit = self.credential_limiter.acquire_register(&command.email)?;
        use_case::resend_verification::execute(
            &self.pool,
            &self.verification_key,
            &self.verification_mailer,
            command,
        )
        .await
    }

    pub(crate) async fn verify_email(&self, command: VerifyEmail) -> AppResult<IssuedSession> {
        let _permit = self.credential_limiter.acquire_register(&command.email)?;
        use_case::verify_email::execute(
            &self.pool,
            self.session_ttl,
            &self.verification_key,
            command,
        )
        .await
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
    ) -> AppResult<IssuedSession> {
        let _permit = self
            .credential_limiter
            .acquire_password(session.account_id)?;
        use_case::change_password::execute(&self.pool, self.session_ttl, session, command).await
    }

    /// 使用 Cookie 中的原始令牌完成会话鉴权。
    pub async fn authenticate(&self, raw_token: &str) -> AppResult<AuthenticatedSession> {
        self.authenticate_with_metadata(raw_token)
            .await
            .map(|(session, _)| session)
    }

    pub(crate) async fn authenticate_with_metadata(
        &self,
        raw_token: &str,
    ) -> AppResult<(AuthenticatedSession, SessionMetadata)> {
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
