//! 接收创建版本请求并返回新草稿。

use axum::{Json, extract::Extension, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{CreateReleaseInput, Release, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateReleaseInput>,
) -> AppResult<(StatusCode, Json<Release>)> {
    let actor = AdminActor::from_session(&session)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_release(&actor, input).await?),
    ))
}
