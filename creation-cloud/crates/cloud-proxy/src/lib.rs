//! 提供官方代理节点目录、目标绑定短凭据和独立内网鉴权路由。

mod crypto;
mod error;
mod handler;
mod model;
mod repository;
mod service;
mod validation;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post},
};

pub use error::ProxyError;
pub use model::{
    AuthorizeRequest, CloseTunnelRequest, CredentialIssueRequest, CredentialIssueResponse,
    NodeListResponse, TargetProtocol,
};
pub use service::Service;

const MAX_REQUEST_BYTES: usize = 8 * 1024;

#[must_use = "公网代理路由必须挂载到受认证的 /api/v1/proxy"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route("/nodes", get(handler::list_nodes))
        .route("/credentials", post(handler::issue_credential))
        .route(
            "/credentials/{credential_id}",
            delete(handler::revoke_credential),
        )
        .with_state(service)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .layer(middleware::from_fn(handler::no_store))
}

/// 此路由只允许挂在独立回环监听，禁止合并进公网应用 Router。
#[must_use = "内网鉴权路由必须挂载到独立回环监听"]
pub fn internal_router(service: Service) -> Router {
    Router::new()
        .route("/internal/v1/proxy/authorize", post(handler::authorize))
        .route("/internal/v1/proxy/close", post(handler::close_tunnel))
        .with_state(service)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .layer(middleware::from_fn(handler::no_store))
}

#[cfg(test)]
mod tests;
