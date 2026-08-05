//! 提供账号认证、Cookie 会话和跨业务路由复用的鉴权中间件。

mod captcha;
mod cookie;
mod credential_limiter;
mod handler;
mod login_limiter;
mod mailer;
mod middleware;
mod model;
mod password;
mod repository;
mod request_metadata;
mod service;
mod session;
mod token;
mod use_case;
mod validation;
mod verification;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
    routing::{get, post},
};

pub use mailer::{VerificationMailer, VerificationPurpose};
pub use middleware::{
    authenticate_page_session, authenticate_session, require_admin, require_csrf,
    require_page_session, require_session,
};
pub use request_metadata::TrustedRequestMetadata;
pub use service::Service;
pub use session::{AuthenticatedSession, IssuedSession, SessionMetadata, SessionView};
pub use use_case::{
    AuthSettings, ChangePassword, ClientLoginConfig, Login, LoginCaptchaSettings, LoginOutcome,
    LoginVerificationRequired, PasswordResetVerificationRequired, Register, RegistrationOutcome,
    RegistrationStatus, RequestPasswordReset, ResendLoginVerification, ResendStatus,
    ResendVerification, ResetPassword, UpdateAuthSettings, VerifyEmail, VerifyLogin,
};

/// 为带外管理员创建命令生成符合当前账号策略的 Argon2id 哈希。
///
/// 调用方只能持有并传入原始密码，禁止记录、回显或持久化该输入。
pub async fn hash_admin_password(value: &str) -> cloud_domain::AppResult<String> {
    validation::password(value)?;
    password::hash(value.to_owned()).await
}

/// 构建不含业务前缀的认证路由，由服务端统一挂载。
#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    let protected = Router::new()
        .route("/session", get(handler::session::handle))
        .route("/logout", post(handler::logout::handle))
        .route("/change-password", post(handler::change_password::handle))
        .route_layer(axum_middleware::from_fn_with_state(
            service.clone(),
            require_session,
        ));

    Router::new()
        .route("/captcha", get(handler::captcha::handle))
        .route("/register", post(handler::register::handle))
        .route("/verify-email", post(handler::verify_email::handle))
        .route(
            "/resend-verification",
            post(handler::resend_verification::handle),
        )
        .route("/login", post(handler::login::handle))
        .route("/verify-login", post(handler::verify_login::handle))
        .route(
            "/resend-login-verification",
            post(handler::resend_login_verification::handle),
        )
        .route(
            "/password-reset/request",
            post(handler::request_password_reset::handle),
        )
        .route(
            "/password-reset/confirm",
            post(handler::reset_password::handle),
        )
        .merge(protected)
        .with_state(service)
        .layer(DefaultBodyLimit::max(4 * 1024))
}

/// 构建浏览器表单专用路由，成功后跳转到用户中心。
#[must_use = "路由必须挂载到服务端才会生效"]
pub fn form_router(service: Service) -> Router {
    Router::new()
        .route("/register", post(handler::form_register::handle))
        .route("/login", post(handler::form_login::handle))
        .route("/verify-email", post(handler::form_verify_email::handle))
        .route("/verify-login", post(handler::form_verify_login::handle))
        .route(
            "/resend-verification",
            post(handler::form_resend_verification::handle),
        )
        .route(
            "/resend-login-verification",
            post(handler::form_resend_login_verification::handle),
        )
        .route(
            "/password-reset/request",
            post(handler::form_request_password_reset::handle),
        )
        .route(
            "/password-reset/confirm",
            post(handler::form_reset_password::handle),
        )
        .with_state(service)
        .layer(DefaultBodyLimit::max(4 * 1024))
}

#[cfg(test)]
mod tests;
