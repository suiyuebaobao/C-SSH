//! 分别组装用户反馈与管理反馈相对路由，最终前缀和认证由服务端负责。

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, patch, post},
};

use crate::{Service, handler};

const JSON_BODY_LIMIT: usize = 16 * 1024;

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn user_router(service: Service) -> Router {
    Router::new()
        .route(
            "/",
            post(handler::user::create::handle).get(handler::user::list::handle),
        )
        .route("/{id}", get(handler::user::get::handle))
        .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT))
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(handler::management::list::handle))
        .route("/{id}", get(handler::management::get::handle))
        .route("/{id}/status", patch(handler::management::status::handle))
        .route("/{id}/redact", post(handler::management::redact::handle))
        .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT))
        .with_state(service)
}
