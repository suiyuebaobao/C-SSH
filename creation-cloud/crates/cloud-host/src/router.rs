//! Relative Axum routers for account hosts, device policy, sync, and admin reads.

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{
    HostConflictView, HostDownloadAllowlist, HostView, PullAckRequest, PullRequest, PullResponse,
    PushOutcome, PushRequest, ReplaceAllowlistRequest, ResolveConflictOutcome,
    ResolveConflictRequest, Service,
};

#[must_use = "the router must be mounted by the server"]
pub fn router(service: Service) -> Router {
    Router::new()
        .nest("/hosts", host_router(service.clone()))
        .nest("/devices", device_router(service.clone()))
        .nest("/sync", sync_router(service))
}

#[must_use = "mount below /api/v1/hosts"]
pub fn host_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_self))
        .route("/{host_id}", get(get_self))
        .with_state(service)
}

#[must_use = "merge below /api/v1/devices"]
pub fn device_router(service: Service) -> Router {
    Router::new()
        .route(
            "/{device_id}/host-download-allowlist",
            get(get_allowlist).put(replace_allowlist),
        )
        .with_state(service)
}

#[must_use = "mount below /api/v1/sync"]
pub fn sync_router(service: Service) -> Router {
    Router::new()
        .route("/push", post(push))
        .route("/pull", get(pull))
        .route("/pull/ack", post(ack_pull))
        .route("/conflicts", get(list_conflicts))
        .route("/conflicts/{conflict_id}", get(get_conflict))
        .route("/conflicts/{conflict_id}/resolve", post(resolve_conflict))
        .with_state(service)
}

/// Merge below `/api/v1/admin/users`.
///
/// Every route requires an explicit `account_id`; there is intentionally no
/// global administrator host-list endpoint.
#[must_use = "merge below /api/v1/admin/users"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/{account_id}/hosts", get(admin_list_for_user))
        .route("/{account_id}/hosts/{host_id}", get(admin_get_for_user))
        .with_state(service)
}

async fn list_self(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<HostView>>> {
    service.list_self(&session, page).await.map(Json)
}

async fn get_self(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(host_id): Path<Uuid>,
) -> AppResult<Json<HostView>> {
    service.get_self(&session, host_id).await.map(Json)
}

async fn get_allowlist(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
) -> AppResult<Json<HostDownloadAllowlist>> {
    service
        .get_download_allowlist(&session, device_id)
        .await
        .map(Json)
}

async fn replace_allowlist(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    Json(request): Json<ReplaceAllowlistRequest>,
) -> AppResult<Json<HostDownloadAllowlist>> {
    service
        .replace_download_allowlist(&session, device_id, request)
        .await
        .map(Json)
}

async fn push(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<PushRequest>,
) -> AppResult<Response> {
    let outcome = service.push(&session, request).await?;
    let status = if matches!(outcome, PushOutcome::Conflict { .. }) {
        StatusCode::CONFLICT
    } else {
        StatusCode::OK
    };
    Ok((status, Json(outcome)).into_response())
}

async fn pull(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(request): Query<PullRequest>,
) -> AppResult<Json<PullResponse>> {
    service.pull(&session, request).await.map(Json)
}

async fn ack_pull(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<PullAckRequest>,
) -> AppResult<StatusCode> {
    service.acknowledge_pull(&session, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_conflicts(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<HostConflictView>>> {
    service.list_open_conflicts(&session, page).await.map(Json)
}

async fn get_conflict(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(conflict_id): Path<Uuid>,
) -> AppResult<Json<HostConflictView>> {
    service.get_conflict(&session, conflict_id).await.map(Json)
}

async fn resolve_conflict(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(conflict_id): Path<Uuid>,
    Json(request): Json<ResolveConflictRequest>,
) -> AppResult<Json<ResolveConflictOutcome>> {
    service
        .resolve_conflict(&session, conflict_id, request)
        .await
        .map(Json)
}

async fn admin_list_for_user(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<HostView>>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .admin_list_for_user(&actor, account_id, page)
        .await
        .map(Json)
}

async fn admin_get_for_user(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((account_id, host_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<HostView>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .admin_get_for_user(&actor, account_id, host_id)
        .await
        .map(Json)
}
