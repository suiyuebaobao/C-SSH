//! 处理浏览器登录验证码重发，并跳转到新的不透明挑战。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;
use uuid::Uuid;

use crate::{ResendLoginVerification, Service};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserResendLoginVerification {
    challenge_id: Uuid,
    next: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserResendLoginVerification>,
) -> AppResult<Response> {
    let status = service
        .resend_login_verification(ResendLoginVerification {
            challenge_id: form.challenge_id,
        })
        .await?;
    let destination = form_response::safe_destination(form.next.as_deref());
    form_response::redirect_without_session(
        &headers,
        &form_response::login_verification_destination(
            form.lang.as_deref() == Some("en"),
            status.challenge_id,
            Some(destination),
            true,
        ),
    )
}
