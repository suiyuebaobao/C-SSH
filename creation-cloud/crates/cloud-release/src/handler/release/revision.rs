//! 接收显式同版本修订请求，沿统一管理路由的认证与 CSRF 边界执行。

use crate::{CreateRevisionInput, Release, Service};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateRevisionInput>,
) -> AppResult<(StatusCode, Json<Release>)> {
    let actor = AdminActor::from_session(&session)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_revision(&actor, id, input).await?),
    ))
}
