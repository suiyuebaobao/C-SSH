//! 提供账号主机、统一密文同步和管理员只读路由。

use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{
    AdminSyncRecord, HostView, PullAckRequest, PullRequest, PullResponse, PushOutcome, PushRequest,
    RekeySyncRequest, RekeySyncResponse, ResetSyncRequest, ResetSyncResponse, Service,
    SyncStateView,
};

/// 32 MiB decoded ciphertext expands to at most 42.67 MiB of canonical
/// Base64. The remaining space covers the bounded opaque envelope and JSON.
pub(crate) const SYNC_WRITE_REQUEST_BODY_LIMIT_BYTES: usize = 45 * 1024 * 1024;
pub(crate) const SYNC_ACK_REQUEST_BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;

#[must_use = "the router must be mounted by the server"]
pub fn router(service: Service) -> Router {
    Router::new()
        .nest("/hosts", host_router(service.clone()))
        .nest("/sync", sync_router(service))
}

#[must_use = "mount below /api/v1/hosts"]
pub fn host_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_self))
        .route("/{host_id}", get(get_self))
        .with_state(service)
}

#[must_use = "mount below /api/v1/sync"]
pub fn sync_router(service: Service) -> Router {
    Router::new()
        .route("/state", get(sync_state))
        .route(
            "/push",
            post(push).layer(DefaultBodyLimit::max(SYNC_WRITE_REQUEST_BODY_LIMIT_BYTES)),
        )
        .route("/pull", get(pull))
        .route(
            "/pull/ack",
            post(ack_pull).layer(DefaultBodyLimit::max(SYNC_ACK_REQUEST_BODY_LIMIT_BYTES)),
        )
        .route(
            "/rekey",
            post(rekey_sync).layer(DefaultBodyLimit::max(SYNC_WRITE_REQUEST_BODY_LIMIT_BYTES)),
        )
        .route("/reset", post(reset_sync))
        .with_state(service)
}

async fn sync_state(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<SyncStateView>> {
    service.sync_state(&session).await.map(Json)
}

/// Merge below `/api/v1/admin/users`.
///
/// Every route requires an explicit `account_id`; there is intentionally no
/// global administrator host-list endpoint.
#[must_use = "merge below /api/v1/admin/users"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/{account_id}/hosts", get(admin_list_for_user))
        .route(
            "/{account_id}/hosts/{host_id}",
            get(admin_get_for_user).delete(admin_delete_for_user),
        )
        .route("/{account_id}/sync-records", get(admin_list_sync_records))
        .route(
            "/{account_id}/sync-records/{record_id}",
            delete(admin_delete_sync_record),
        )
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

async fn push(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<PushRequest>,
) -> AppResult<Json<PushOutcome>> {
    service.push(&session, request).await.map(Json)
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

async fn reset_sync(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<ResetSyncRequest>,
) -> AppResult<Json<ResetSyncResponse>> {
    service.reset_sync(&session, request).await.map(Json)
}

async fn rekey_sync(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<RekeySyncRequest>,
) -> AppResult<Json<RekeySyncResponse>> {
    service.rekey_sync(&session, request).await.map(Json)
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

async fn admin_delete_for_user(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((account_id, host_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service
        .admin_delete_for_user(&actor, account_id, host_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn admin_list_sync_records(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<AdminSyncRecord>>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .admin_list_sync_records(&actor, account_id, page)
        .await
        .map(Json)
}

async fn admin_delete_sync_record(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((account_id, record_id)): Path<(Uuid, String)>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service
        .admin_delete_sync_record(&actor, account_id, &record_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
