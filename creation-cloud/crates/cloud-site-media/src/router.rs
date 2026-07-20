//! 分离组装站点媒体管理写路由与公开只读路由。

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{Service, handler};

const MULTIPART_BODY_LIMIT: usize = 2 * 1024 * 1024 + 64 * 1024;

#[must_use = "路由必须挂载到服务端并叠加管理员中间件才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route(
            "/",
            get(handler::management::list::handle).post(handler::management::create::handle),
        )
        .route(
            "/{media_id}",
            get(handler::management::get::handle)
                .patch(handler::management::update::handle)
                .delete(handler::management::delete::handle),
        )
        .route(
            "/{media_id}/publish",
            post(handler::management::publish::handle),
        )
        .route(
            "/{media_id}/revoke",
            post(handler::management::revoke::handle),
        )
        .layer(DefaultBodyLimit::max(MULTIPART_BODY_LIMIT))
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn public_router(service: Service) -> Router {
    Router::new()
        .route("/home-qr", get(handler::public::current::handle))
        .route("/{media_id}/content", get(handler::public::content::handle))
        .with_state(service)
}
