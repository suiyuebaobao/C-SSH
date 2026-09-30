//! 处理代理控制面的公网会话请求和独立监听上的节点鉴权请求。

use axum::{
    Extension, Json,
    extract::{
        Path, Query, Request, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::{
    ProxyError, Service,
    model::{AuthorizeRequest, CloseTunnelRequest, CredentialIssueRequest, NodeListQuery},
    validation,
};

pub(crate) async fn list_nodes(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    query: Result<Query<NodeListQuery>, QueryRejection>,
) -> Result<Json<crate::NodeListResponse>, ProxyError> {
    let Query(query) = query.map_err(|_| ProxyError::InvalidRequest)?;
    service
        .list_nodes(&session, query.target_protocol)
        .await
        .map(Json)
}

pub(crate) async fn issue_credential(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    request: Result<Json<CredentialIssueRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<crate::CredentialIssueResponse>), ProxyError> {
    let Json(request) = request.map_err(|_| ProxyError::InvalidRequest)?;
    let response = service.issue_credential(&session, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub(crate) async fn revoke_credential(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(credential_id): Path<String>,
) -> Result<StatusCode, ProxyError> {
    let credential_id = Uuid::parse_str(&credential_id).map_err(|_| ProxyError::InvalidRequest)?;
    service.revoke_credential(&session, credential_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn authorize(
    State(service): State<Service>,
    headers: HeaderMap,
    request: Result<Json<AuthorizeRequest>, JsonRejection>,
) -> Result<Json<impl serde::Serialize>, ProxyError> {
    let raw_authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ProxyError::AuthorizationDenied)?;
    let token = validation::bearer(raw_authorization)?;
    let Json(request) = request.map_err(|_| ProxyError::AuthorizationDenied)?;
    service.authorize(token, request).await.map(Json)
}

pub(crate) async fn close_tunnel(
    State(service): State<Service>,
    headers: HeaderMap,
    request: Result<Json<CloseTunnelRequest>, JsonRejection>,
) -> Result<StatusCode, ProxyError> {
    let raw_authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ProxyError::AuthorizationDenied)?;
    let token = validation::bearer(raw_authorization)?;
    let Json(request) = request.map_err(|_| ProxyError::AuthorizationDenied)?;
    service.close_tunnel(token, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn no_store(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
        .headers_mut()
        .insert(header::PRAGMA, header::HeaderValue::from_static("no-cache"));
    response
}
