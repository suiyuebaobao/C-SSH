//! 分别组装公开分发与来源管理路由，防止写接口进入公开路由树。

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{
    Service,
    handler::{account, public, source, upload},
};

#[must_use = "路由必须挂载到已注入认证会话的服务端才会生效"]
pub fn account_router(service: Service) -> Router {
    Router::new()
        .route("/history", get(account::history::handle))
        .route(
            "/account/assets/{asset_id}/sources/{source_id}",
            get(account::download::handle),
        )
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route(
            "/assets/{asset_id}/sources",
            get(source::list::handle).post(source::create::handle),
        )
        .route(
            "/assets/{asset_id}/upload",
            post(upload::handle).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/sources/{source_id}",
            get(source::get::handle)
                .patch(source::update::handle)
                .delete(source::delete::handle),
        )
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn public_router(service: Service) -> Router {
    Router::new()
        .route("/releases", get(public::manifest::handle))
        .route(
            "/assets/{asset_id}/sources/{source_id}",
            get(public::download::handle),
        )
        .with_state(service)
}
