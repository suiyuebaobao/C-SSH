//! 组装公开播放与管理员CRUD；写路由由server统一叠加认证、CSRF及审计链。

use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{
    Service, content,
    model::{AdminVideo, ListQuery, PublicVideo, TransitionInput, UpdateInput},
    multipart, repository, storage, validation,
};

pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(admin_list).post(create))
        .route("/{id}", get(admin_get).patch(update).delete(delete))
        .route("/{id}/publish", post(publish))
        .route("/{id}/hide", post(hide))
        .route("/{id}/video", get(admin_video))
        .route("/{id}/poster", get(admin_poster))
        .layer(DefaultBodyLimit::max(204 * 1024 * 1024))
        .with_state(service)
}

pub fn public_router(service: Service) -> Router {
    Router::new()
        .route("/", get(public_list))
        .route("/{id}/video", get(public_video))
        .route("/{id}/poster", get(public_poster))
        .with_state(service)
}

async fn public_list(
    State(service): State<Service>,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    if let Some(ref locale) = query.locale {
        validation::locale(locale)?;
    }
    let rows = repository::list(
        &service.pool,
        query.locale.as_deref(),
        query.offset.unwrap_or(0),
        true,
    )
    .await?;
    Ok((
        [("cache-control", "no-store")],
        Json(
            rows.iter()
                .map(|row| row.public_view())
                .collect::<Vec<PublicVideo>>(),
        ),
    ))
}

async fn admin_list(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<AdminVideo>>> {
    AdminActor::from_session(&session)?;
    if let Some(ref locale) = query.locale {
        validation::locale(locale)?;
    }
    let rows = repository::list(
        &service.pool,
        query.locale.as_deref(),
        query.offset.unwrap_or(0),
        false,
    )
    .await?;
    Ok(Json(rows.iter().map(|row| row.admin_view()).collect()))
}

async fn admin_get(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AdminVideo>> {
    AdminActor::from_session(&session)?;
    validation::id(id)?;
    Ok(Json(repository::get(&service.pool, id).await?.admin_view()))
}

async fn create(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<AdminVideo>)> {
    let actor = AdminActor::from_session(&session)?;
    let _permit = service
        .uploads
        .clone()
        .try_acquire_owned()
        .map_err(|_| AppError::RateLimited("正在处理其他教学上传，请稍后提交".into()))?;
    let root = storage::prepare(&service.media_base).await?;
    let upload = multipart::receive(&root, multipart).await?;
    Ok((
        StatusCode::CREATED,
        Json(service.create(actor, root, upload).await?.admin_view()),
    ))
}

async fn update(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateInput>,
) -> AppResult<Json<AdminVideo>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service
            .update(actor, id, input.expected_revision, input.metadata)
            .await?
            .admin_view(),
    ))
}

async fn publish(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<TransitionInput>,
) -> AppResult<Json<AdminVideo>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service
            .transition(actor, id, input.expected_revision, "published")
            .await?
            .admin_view(),
    ))
}

async fn hide(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<TransitionInput>,
) -> AppResult<Json<AdminVideo>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service
            .transition(actor, id, input.expected_revision, "hidden")
            .await?
            .admin_view(),
    ))
}

async fn delete(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<TransitionInput>,
) -> AppResult<StatusCode> {
    service
        .delete(
            AdminActor::from_session(&session)?,
            id,
            input.expected_revision,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn public_video(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Response> {
    content::response(&service, id, false, true, headers).await
}
async fn public_poster(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Response> {
    content::response(&service, id, true, true, headers).await
}
async fn admin_video(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Response> {
    AdminActor::from_session(&session)?;
    content::response(&service, id, false, false, headers).await
}
async fn admin_poster(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Response> {
    AdminActor::from_session(&session)?;
    content::response(&service, id, true, false, headers).await
}
