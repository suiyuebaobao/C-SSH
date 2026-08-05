//! Starts browser password recovery, stores the normalized email briefly, and redirects.

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;

use crate::{RequestPasswordReset, Service, captcha::CaptchaPurpose, cookie, validation};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserRequestPasswordReset {
    email: String,
    lang: Option<String>,
    captcha_code: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserRequestPasswordReset>,
) -> AppResult<Response> {
    let is_en = form.lang.as_deref() == Some("en");
    let email = validation::normalize_email(&form.email)?;
    service
        .request_password_reset(RequestPasswordReset {
            email: email.clone(),
            captcha_id: cookie::read_captcha(&headers, CaptchaPurpose::PasswordReset),
            captcha_code: form.captcha_code,
        })
        .await?;

    let mut response = form_response::redirect_without_session(
        &headers,
        if is_en {
            "/en/reset-password"
        } else {
            "/reset-password"
        },
    )?;
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        cookie::password_reset_email_header(&email)?,
    );
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        cookie::clear_captcha_header(CaptchaPurpose::PasswordReset)?,
    );
    Ok(response)
}
