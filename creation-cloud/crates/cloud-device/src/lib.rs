//! 提供账号设备登记、查询、重命名和撤销的独立服务与相对路由。

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

pub use model::{CreateDeviceOutcome, Device, Platform};
pub use service::Service;
pub use use_case::{CreateDevice, UpdateDevice};

/// 构建不含 `/api/v1/devices` 前缀的设备路由。
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
