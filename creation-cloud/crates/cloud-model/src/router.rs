//! 用户只读目录、个人密文与管理员模型路由。

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{
    CreateGlobalModelInput, DeleteGlobalModelInput, DeleteModelSecretInput, GlobalModel,
    ModelSecret, PutModelSecretInput, ReplaceGlobalModelInput, Service,
};

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_public))
        .route("/{id}", get(get_public))
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn secret_router(service: Service) -> Router {
    Router::new()
        .route(
            "/{model_id}",
            get(get_secret).put(put_secret).delete(delete_secret),
        )
        .with_state(service)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_admin).post(create_admin))
        .route(
            "/{id}",
            get(get_admin).patch(replace_admin).delete(delete_admin),
        )
        .with_state(service)
}

async fn list_public(
    State(service): State<Service>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<GlobalModel>>> {
    service.list_public(page).await.map(Json)
}

async fn get_public(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<GlobalModel>> {
    service.get_public(id).await.map(Json)
}

async fn list_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<GlobalModel>>> {
    let actor = AdminActor::from_session(&session)?;
    service.list_admin(&actor, page).await.map(Json)
}

async fn get_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<GlobalModel>> {
    let actor = AdminActor::from_session(&session)?;
    service.get_admin(&actor, id).await.map(Json)
}

async fn create_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateGlobalModelInput>,
) -> AppResult<(StatusCode, Json<GlobalModel>)> {
    let actor = AdminActor::from_session(&session)?;
    service
        .create_admin(&actor, input)
        .await
        .map(|model| (StatusCode::CREATED, Json(model)))
}

async fn replace_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<ReplaceGlobalModelInput>,
) -> AppResult<Json<GlobalModel>> {
    let actor = AdminActor::from_session(&session)?;
    service.replace_admin(&actor, id, input).await.map(Json)
}

async fn delete_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<DeleteGlobalModelInput>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service
        .delete_admin(&actor, id, input.expected_revision)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_secret(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
) -> AppResult<Json<ModelSecret>> {
    service.get_secret(&session, model_id).await.map(Json)
}

async fn put_secret(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
    Json(input): Json<PutModelSecretInput>,
) -> AppResult<Json<ModelSecret>> {
    service
        .put_secret(
            &session,
            model_id,
            &input.ciphertext,
            input.expected_revision,
        )
        .await
        .map(Json)
}

async fn delete_secret(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
    Json(input): Json<DeleteModelSecretInput>,
) -> AppResult<Json<ModelSecret>> {
    service
        .delete_secret(&session, model_id, input.expected_revision)
        .await
        .map(Json)
}
