//! 组装管理用户、设备、概览与只读审计相对路由，并自动审计写请求。

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    Service,
    handler::{audit, devices, overview, users},
    middleware::audit::audit_write_requests,
};

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    finish(
        routes().route("/overview", get(overview::all::handle)),
        service,
    )
}

/// 为服务端管理编排层保留组合概览和统一审计，其余管理资源仍由本领域拥有。
#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router_without_overview(service: Service) -> Router {
    routes().with_state(service)
}

fn routes() -> Router<Service> {
    Router::new()
        .route("/overview/users", get(overview::users::handle))
        .route("/overview/devices", get(overview::devices::handle))
        .route("/overview/releases", get(overview::releases::handle))
        .route("/overview/audit", get(overview::audit::handle))
        .route("/users", get(users::list::handle))
        .route(
            "/users/{account_id}",
            get(users::get::handle).patch(users::update::handle),
        )
        .route("/devices", get(devices::list::handle))
        .route("/devices/{device_id}", get(devices::get::handle))
        .route("/devices/{device_id}/revoke", post(devices::revoke::handle))
        .merge(audit_routes())
}

fn finish(router: Router<Service>, service: Service) -> Router {
    router
        .route_layer(middleware::from_fn_with_state(
            service.clone(),
            audit_write_requests,
        ))
        .with_state(service)
}

pub(crate) fn audit_routes() -> Router<Service> {
    Router::new()
        .route("/audit-events", get(audit::list::handle))
        .route("/audit-events/{event_id}", get(audit::get::handle))
}
