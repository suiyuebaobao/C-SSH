//! 装配同步领域的相对 Axum 路由，最终前缀由 server 统一添加。

use axum::{
    Router,
    routing::{get, post},
};

use crate::{Service, handler};

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route("/push", post(handler::push))
        .route("/pull", get(handler::pull))
        .route("/conflicts", get(handler::list_conflicts))
        .route("/conflicts/{id}", get(handler::get_conflict))
        .route("/conflicts/{id}/resolve", post(handler::resolve_conflict))
        .with_state(service)
}
