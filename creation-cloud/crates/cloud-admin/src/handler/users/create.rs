//! 将管理员创建账号 JSON 映射到完整创建用例。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{AdminCreateUserInput, AdminUser, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<AdminCreateUserInput>,
) -> AppResult<(StatusCode, Json<AdminUser>)> {
    let actor = AdminActor::from_session(&session)?;
    let user = service.create_user(&actor, input).await?;
    Ok((StatusCode::CREATED, Json(user)))
}
