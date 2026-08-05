//! 使用不透明挑战标识重发普通用户或管理员的登录邮箱验证码。

use axum::{Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;

use crate::{LoginVerificationRequired, ResendLoginVerification, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<ResendLoginVerification>,
) -> AppResult<(StatusCode, Json<LoginVerificationRequired>)> {
    let status = service.resend_login_verification(command).await?;
    Ok((StatusCode::ACCEPTED, Json(status)))
}
