use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{
    CreateSiteContentInput, Service, SiteContentListQuery, SiteContentRevision,
    SiteContentTransitionInput, UpdateSiteContentInput,
};

#[must_use = "路由必须挂载到已有管理员认证链后才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route(
            "/{content_id}",
            get(get_one).patch(update).delete(delete_draft),
        )
        .route("/{content_id}/preview", axum::routing::post(preview))
        .route("/{content_id}/publish", axum::routing::post(publish))
        .route("/{content_id}/revoke", axum::routing::post(revoke))
        .route("/{content_id}/rollback", axum::routing::post(rollback))
        .with_state(service)
}

async fn list(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SiteContentListQuery>,
) -> AppResult<Json<Vec<SiteContentRevision>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list(&actor, query).await?))
}

async fn get_one(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get(&actor, content_id).await?))
}

async fn create(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateSiteContentInput>,
) -> AppResult<(StatusCode, Json<SiteContentRevision>)> {
    let actor = AdminActor::from_session(&session)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_draft(&actor, input).await?),
    ))
}

async fn update(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    Json(input): Json<UpdateSiteContentInput>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update_draft(&actor, content_id, input).await?))
}

async fn delete_draft(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    Json(input): Json<SiteContentTransitionInput>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_draft(&actor, content_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn preview(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get(&actor, content_id).await?))
}

async fn publish(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    Json(input): Json<SiteContentTransitionInput>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.publish(&actor, content_id, input).await?))
}

async fn revoke(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    Json(input): Json<SiteContentTransitionInput>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.revoke(&actor, content_id, input).await?))
}

async fn rollback(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    Json(input): Json<SiteContentTransitionInput>,
) -> AppResult<Json<SiteContentRevision>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.rollback(&actor, content_id, input).await?))
}
