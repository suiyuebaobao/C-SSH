//! 装配保险库领域的相对 Axum 路由，最终前缀由 server 统一添加。

use axum::{Router, routing::get};

use crate::{Service, handler};

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route("/envelopes", get(handler::list).post(handler::create))
        .route(
            "/envelopes/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .route(
            "/wrappers",
            get(handler::wrapper::list::handle).post(handler::wrapper::create::handle),
        )
        .route(
            "/wrappers/{id}",
            get(handler::wrapper::get::handle)
                .put(handler::wrapper::update::handle)
                .delete(handler::wrapper::delete::handle),
        )
        .with_state(service)
}
