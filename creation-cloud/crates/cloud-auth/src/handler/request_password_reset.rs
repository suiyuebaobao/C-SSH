//! Starts an anonymous, non-enumerating password-reset challenge.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{RequestPasswordReset, Service, captcha::CaptchaPurpose, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(mut command): Json<RequestPasswordReset>,
) -> AppResult<Response> {
    if command.captcha_id.is_none() {
        command.captcha_id = cookie::read_captcha(&headers, CaptchaPurpose::PasswordReset);
    }
    let status = service.request_password_reset(command).await?;
    let mut response = (StatusCode::ACCEPTED, Json(status)).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        cookie::clear_captcha_header(CaptchaPurpose::PasswordReset)?,
    );
    Ok(response)
}
