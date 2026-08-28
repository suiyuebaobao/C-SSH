//! 分别组装公开分发与来源管理路由，防止写接口进入公开路由树。

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post, put},
};

use crate::{
    Service,
    handler::{account, installed_identity, policy, public, signature, source, upload},
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
        .route("/policy", get(policy::get))
        .route("/policy/draft", put(policy::save_draft))
        .route("/policy/publish", post(policy::publish))
        .route(
            "/installed-identities",
            post(installed_identity::handle).layer(DefaultBodyLimit::max(32 * 1024)),
        )
        .route(
            "/assets/{asset_id}/sources",
            get(source::list::handle).post(source::create::handle),
        )
        .route(
            "/assets/{asset_id}/upload",
            post(upload::handle).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/assets/{asset_id}/updater-signature",
            post(signature::upload).layer(DefaultBodyLimit::max(16 * 1024)),
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

#[must_use = "路由必须以 /updates 挂载到匿名公开 API 才会生效"]
pub fn update_router(service: Service) -> Router {
    Router::new()
        .route("/check", get(public::update::handle))
        .route("/tauri", get(public::tauri::handle))
        .with_state(service)
}
