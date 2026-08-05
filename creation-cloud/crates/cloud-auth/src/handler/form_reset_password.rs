//! Confirms a browser password reset using the short-lived email cookie.

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppError, AppResult};
use serde::Deserialize;

use crate::{ResetPassword, Service, cookie};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserResetPassword {
    code: String,
    new_password: String,
    confirm_password: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserResetPassword>,
) -> AppResult<Response> {
    let is_en = form.lang.as_deref() == Some("en");
    if form.new_password != form.confirm_password {
        return Err(AppError::Validation(if is_en {
            "The two new-password values do not match".to_owned()
        } else {
            "两次输入的新密码不一致".to_owned()
        }));
    }
    let email = cookie::read_password_reset_email(&headers).ok_or_else(|| {
        AppError::Unauthorized(if is_en {
            "Password reset verification is invalid or expired".to_owned()
        } else {
            "密码重置验证无效或已过期".to_owned()
        })
    })?;
    let outcome = service
        .reset_password(ResetPassword {
            email,
            code: form.code,
            new_password: form.new_password,
        })
        .await?;

    let destination = match (is_en, outcome.is_admin) {
        (true, true) => "/en/login?reset=1&next=%2Fadmin",
        (true, false) => "/en/login?reset=1",
        (false, true) => "/login?reset=1&next=%2Fadmin",
        (false, false) => "/login?reset=1",
    };
    let mut response = form_response::redirect_without_session(&headers, destination)?;
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        cookie::clear_password_reset_email_header()?,
    );
    Ok(response)
}
