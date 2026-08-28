//! 提供账号设备登记、查询、重命名和撤销的独立服务与相对路由。

mod handler;
mod model;
mod repository;
mod service;
mod session;
mod use_case;
mod validation;

use axum::{
    Router,
    routing::{delete, get, post},
};

pub use model::{
    CreateDeviceOutcome, Device, DeviceSessionResult, DeviceSessionView, Platform, SessionView,
};
pub use service::Service;
pub use use_case::{CreateDevice, UpdateDevice};

/// Hard per-account bound for devices that can retain an active device session.
///
/// Device creation serializes on the owning account row before enforcing this
/// limit, so concurrent registrations cannot exceed it.
pub const MAX_ACTIVE_DEVICES_PER_ACCOUNT: i64 = 16;

/// 构建不含 `/api/v1/devices` 前缀的设备路由。
#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route("/sessions", get(handler::session::list_self))
        .route(
            "/sessions/{session_id}",
            delete(handler::session::revoke_self),
        )
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

/// Builds administrator session-management routes without an API prefix.
#[must_use = "the router must be mounted for session management to be available"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/sessions", get(handler::session::list_admin))
        .route(
            "/sessions/{session_id}",
            delete(handler::session::delete_admin),
        )
        .route(
            "/users/{account_id}/sessions",
            get(handler::session::list_admin_for_user),
        )
        .with_state(service)
}

#[cfg(test)]
mod tests;
