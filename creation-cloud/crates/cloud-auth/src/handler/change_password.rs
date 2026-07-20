//! 接收改密 JSON，并由用例校验当前密码及撤销策略。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;

use crate::{AuthenticatedSession, ChangePassword, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(command): Json<ChangePassword>,
) -> AppResult<StatusCode> {
    service.change_password(&session, command).await?;
    Ok(StatusCode::NO_CONTENT)
}
