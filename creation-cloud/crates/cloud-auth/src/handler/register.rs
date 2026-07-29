//! 创建待验证账号；成功响应不包含会话 Cookie。

use axum::{Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;

use crate::{Register, RegistrationStatus, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<Register>,
) -> AppResult<(StatusCode, Json<RegistrationStatus>)> {
    let status = service.register(command).await?;
    Ok((StatusCode::ACCEPTED, Json(status)))
}
