//! 持有认证域数据库连接并作为各认证用例的统一调用入口。

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use cloud_domain::{AdminActor, AppResult};
use cloud_store::PgPool;
use rand::RngCore;
use sqlx::PgConnection;

use crate::{
    credential_limiter::CredentialLimiter,
    login_limiter::LoginLimiter,
    mailer::{UnavailableVerificationMailer, VerificationMailer},
    session::{AuthenticatedSession, IssuedSession, SessionMetadata},
    use_case::{
        self, AuthSettings, ChangePassword, ClientLoginConfig, Login, LoginCaptchaSettings,
        LoginOutcome, LoginVerificationRequired, Register, RegistrationOutcome,
        ResendLoginVerification, ResendStatus, ResendVerification, UpdateAuthSettings, VerifyEmail,
        VerifyLogin,
    },
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    session_ttl: Duration,
    verification_key: Arc<[u8]>,
    captcha_key: Arc<[u8]>,
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
            captcha_key: random_captcha_key(),
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
        let captcha_key = if verification_key.len() >= 32 {
            Arc::from(verification_key.clone())
        } else {
            random_captcha_key()
        };
        Self {
            pool,
            session_ttl,
            verification_key: Arc::from(verification_key),
            captcha_key,
            verification_mailer,
            credential_limiter: CredentialLimiter::default(),
            login_limiter: LoginLimiter::default(),
        }
    }

    pub(crate) async fn register(&self, command: Register) -> AppResult<RegistrationOutcome> {
        let _permit = self.credential_limiter.acquire_register(&command.email)?;
        use_case::register::execute(
            &self.pool,
            self.session_ttl,
            &self.verification_key,
            &self.captcha_key,
            &self.verification_mailer,
            command,
        )
        .await
    }

    pub(crate) async fn issue_captcha(
        &self,
        purpose: crate::captcha::CaptchaPurpose,
    ) -> AppResult<use_case::captcha::IssuedCaptcha> {
        use_case::captcha::issue(&self.pool, &self.captcha_key, purpose).await
    }

    pub async fn client_login_config(&self) -> AppResult<ClientLoginConfig> {
        use_case::auth_settings::client_login_config(&self.pool).await
    }

    pub async fn login_captcha_settings(&self) -> AppResult<LoginCaptchaSettings> {
        use_case::auth_settings::login_captcha_settings(&self.pool).await
    }

    pub async fn auth_settings(&self, actor: &AdminActor) -> AppResult<AuthSettings> {
        use_case::auth_settings::get(&self.pool, actor).await
    }

    pub async fn update_auth_settings(
        &self,
        actor: &AdminActor,
        input: UpdateAuthSettings,
    ) -> AppResult<AuthSettings> {
        use_case::auth_settings::update(&self.pool, actor, input).await
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

    pub(crate) async fn login(&self, command: Login) -> AppResult<LoginOutcome> {
        let _permit = self.login_limiter.acquire(&command.identifier)?;
        use_case::login::execute(
            &self.pool,
            self.session_ttl,
            &self.verification_key,
            &self.captcha_key,
            &self.verification_mailer,
            command,
        )
        .await
    }

    pub(crate) async fn verify_login(&self, command: VerifyLogin) -> AppResult<IssuedSession> {
        let _permit = self
            .credential_limiter
            .acquire_login_verification(command.challenge_id)?;
        use_case::verify_login::execute(
            &self.pool,
            self.session_ttl,
            &self.verification_key,
            command,
        )
        .await
    }

    pub(crate) async fn resend_login_verification(
        &self,
        command: ResendLoginVerification,
    ) -> AppResult<LoginVerificationRequired> {
        let _permit = self
            .credential_limiter
            .acquire_login_verification(command.challenge_id)?;
        use_case::resend_login_verification::execute(
            &self.pool,
            &self.verification_key,
            &self.verification_mailer,
            command,
        )
        .await
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

fn random_captcha_key() -> Arc<[u8]> {
    let mut key = [0_u8; 32];
    rand::rng().fill_bytes(&mut key);
    Arc::from(key.to_vec())
}
