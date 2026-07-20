//! 将创建资料请求映射到当前认证账号。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;

use crate::{CreateProfile, Profile, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(command): Json<CreateProfile>,
) -> AppResult<(StatusCode, Json<Profile>)> {
    let profile = service.create(&session, command).await?;
    Ok((StatusCode::CREATED, Json(profile)))
}
