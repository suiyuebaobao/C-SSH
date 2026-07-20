//! 提供当前账号用户资料的独立 CRUD 服务与相对 Axum 路由。

mod handler;
mod model;
mod repository;
mod service;
mod use_case;
mod validation;

use axum::{
    Router,
    routing::{get, post},
};

pub use model::Profile;
pub use service::Service;
pub use use_case::{CreateProfile, UpdateProfile};

/// 构建不含 `/api/v1/users` 前缀的用户资料路由。
#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route(
            "/",
            post(handler::create::handle).get(handler::list::handle),
        )
        .route(
            "/{id}",
            get(handler::get::handle)
                .patch(handler::update::handle)
                .delete(handler::delete::handle),
        )
        .with_state(service)
}

#[cfg(test)]
mod tests;
