use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{
    Announcement, CreateAnnouncementInput, CurrentAnnouncementResponse, ReplaceAnnouncementInput,
    Service, TransitionAnnouncementInput, model::CurrentAnnouncementQuery,
};

#[must_use = "the public announcement router must be mounted to become reachable"]
pub fn public_router(service: Service) -> Router {
    Router::new()
        .route("/current", get(current))
        .with_state(service)
}

#[must_use = "the management router must be mounted behind administrator authentication"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route(
            "/{announcement_id}",
            get(get_one).patch(replace).delete(delete),
        )
        .route("/{announcement_id}/publish", axum::routing::post(publish))
        .route("/{announcement_id}/hide", axum::routing::post(hide))
        .with_state(service)
}

async fn current(
    State(service): State<Service>,
    Query(query): Query<CurrentAnnouncementQuery>,
) -> AppResult<Json<CurrentAnnouncementResponse>> {
    service.current(query.locale).await.map(Json)
}

async fn list(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<Announcement>>> {
    let actor = AdminActor::from_session(&session)?;
    service.list_admin(&actor, page).await.map(Json)
}

async fn get_one(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(announcement_id): Path<Uuid>,
) -> AppResult<Json<Announcement>> {
    let actor = AdminActor::from_session(&session)?;
    service.get_admin(&actor, announcement_id).await.map(Json)
}

async fn create(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateAnnouncementInput>,
) -> AppResult<(StatusCode, Json<Announcement>)> {
    let actor = AdminActor::from_session(&session)?;
    service
        .create_admin(&actor, input)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
}

async fn replace(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(announcement_id): Path<Uuid>,
    Json(input): Json<ReplaceAnnouncementInput>,
) -> AppResult<Json<Announcement>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .replace_admin(&actor, announcement_id, input)
        .await
        .map(Json)
}

async fn delete(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(announcement_id): Path<Uuid>,
    Json(input): Json<TransitionAnnouncementInput>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_admin(&actor, announcement_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn publish(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(announcement_id): Path<Uuid>,
    Json(input): Json<TransitionAnnouncementInput>,
) -> AppResult<Json<Announcement>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .publish_admin(&actor, announcement_id, input)
        .await
        .map(Json)
}

async fn hide(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(announcement_id): Path<Uuid>,
    Json(input): Json<TransitionAnnouncementInput>,
) -> AppResult<Json<Announcement>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .hide_admin(&actor, announcement_id, input)
        .await
        .map(Json)
}
