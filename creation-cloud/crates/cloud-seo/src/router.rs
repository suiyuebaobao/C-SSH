use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{CreateSeoTopicInput, SeoTopic, Service, UpdateSeoTopicInput};

#[must_use = "路由必须挂载到已有管理员认证链后才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{topic_id}", axum::routing::patch(update).delete(delete))
        .with_state(service)
}

async fn list(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<Vec<SeoTopic>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_topics(&actor).await?))
}

async fn create(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateSeoTopicInput>,
) -> AppResult<(StatusCode, Json<SeoTopic>)> {
    let actor = AdminActor::from_session(&session)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_topic(&actor, input).await?),
    ))
}

async fn update(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(topic_id): Path<Uuid>,
    Json(input): Json<UpdateSeoTopicInput>,
) -> AppResult<Json<SeoTopic>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update_topic(&actor, topic_id, input).await?))
}

async fn delete(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(topic_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_topic(&actor, topic_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
