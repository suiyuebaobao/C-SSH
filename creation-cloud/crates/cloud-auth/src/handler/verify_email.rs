//! 验证邮箱后签发短期未绑定会话。

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Service, VerifyEmail, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<VerifyEmail>,
) -> AppResult<Response> {
    let issued = service.verify_email(command).await?;
    let mut response = (StatusCode::CREATED, Json(issued.view())).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
    );
    Ok(response)
}
