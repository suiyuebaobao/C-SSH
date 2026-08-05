//! 接收浏览器登录表单，建立会话后跳转到用户中心。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;

use crate::{Login, LoginOutcome, Service};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserLogin {
    identifier: String,
    password: String,
    next: Option<String>,
    lang: Option<String>,
    captcha_code: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserLogin>,
) -> AppResult<Response> {
    let destination = form_response::safe_destination(form.next.as_deref());
    let purpose = form_response::captcha_purpose_for_destination(destination);
    let command = Login {
        identifier: form.identifier,
        password: form.password,
        captcha_id: crate::cookie::read_captcha(&headers, purpose),
        captcha_code: form.captcha_code,
    };
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    let mut response = match service.login_with_metadata(command, &metadata).await? {
        LoginOutcome::Session(issued) => form_response::redirect(
            &headers,
            &issued.raw_token,
            issued.session.expires_at,
            destination,
        ),
        LoginOutcome::VerificationRequired { status, .. } => {
            form_response::redirect_without_session(
                &headers,
                &form_response::login_verification_destination(
                    form.lang.as_deref() == Some("en"),
                    status.challenge_id,
                    Some(destination),
                    false,
                ),
            )
        }
    }?;
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        crate::cookie::clear_captcha_header(purpose)?,
    );
    Ok(response)
}
