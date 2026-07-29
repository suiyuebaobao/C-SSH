//! Handles browser requests for a fresh email-verification code.

use axum::{
    Form,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};
use serde::Deserialize;

use crate::{ResendVerification, Service};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserResendVerification {
    email: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserResendVerification>,
) -> AppResult<Response> {
    let is_en = form.lang.as_deref() == Some("en");
    service
        .resend_verification(ResendVerification { email: form.email })
        .await?;
    if headers
        .get("hx-request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        let message = if is_en {
            "A new verification code has been sent. Check your inbox."
        } else {
            "新的验证码已发送，请检查邮箱。"
        };
        return Ok((StatusCode::OK, Html(message)).into_response());
    }
    let destination = if is_en {
        "/en/verify-email?resent=1"
    } else {
        "/verify-email?resent=1"
    };
    let mut response = StatusCode::SEE_OTHER.into_response();
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(destination)
            .map_err(|_| AppError::Internal("verification redirect is invalid".to_owned()))?,
    );
    Ok(response)
}
