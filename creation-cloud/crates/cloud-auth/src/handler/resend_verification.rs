//! 接收邮箱验证重发请求并使用通用成功投影。

use axum::{Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;

use crate::{ResendStatus, ResendVerification, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<ResendVerification>,
) -> AppResult<(StatusCode, Json<ResendStatus>)> {
    let status = service.resend_verification(command).await?;
    Ok((StatusCode::ACCEPTED, Json(status)))
}
