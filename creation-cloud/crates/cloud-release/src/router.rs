//! 组装版本与资产管理的相对路由，路径前缀由服务端统一挂载。

use axum::{Router, routing::get};

use crate::{
    Service,
    handler::{asset, release},
};

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route(
            "/",
            get(release::list::handle).post(release::create::handle),
        )
        .route(
            "/{release_id}",
            get(release::get::handle)
                .patch(release::update::handle)
                .delete(release::delete::handle),
        )
        .route(
            "/{release_id}/assets",
            get(asset::list::handle).post(asset::create::handle),
        )
        .route("/assets", get(asset::list_all::handle))
        .route(
            "/assets/{asset_id}",
            get(asset::get::handle)
                .patch(asset::update::handle)
                .delete(asset::delete::handle),
        )
        .with_state(service)
}
