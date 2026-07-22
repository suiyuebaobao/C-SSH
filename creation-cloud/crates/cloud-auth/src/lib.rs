//! 提供账号认证、Cookie 会话和跨业务路由复用的鉴权中间件。

mod cookie;
mod credential_limiter;
mod handler;
mod login_limiter;
mod middleware;
mod model;
mod password;
mod repository;
mod service;
mod session;
mod token;
mod use_case;
mod validation;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
    routing::{get, post},
};

pub use middleware::{
    authenticate_page_session, authenticate_session, require_admin, require_csrf,
    require_page_session, require_session,
};
pub use service::Service;
pub use session::{AuthenticatedSession, SessionView};
pub use use_case::{ChangePassword, Login, Register};

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
        .route("/register", post(handler::register::handle))
        .route("/login", post(handler::login::handle))
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
        .with_state(service)
        .layer(DefaultBodyLimit::max(4 * 1024))
}

#[cfg(test)]
mod tests;
