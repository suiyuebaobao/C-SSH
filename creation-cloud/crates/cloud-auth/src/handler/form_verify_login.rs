//! 处理浏览器登录邮箱验证码并在成功后设置会话 Cookie。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;
use uuid::Uuid;

use crate::{Service, VerifyLogin};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserVerifyLogin {
    challenge_id: Uuid,
    code: String,
    next: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserVerifyLogin>,
) -> AppResult<Response> {
    let destination = form_response::safe_destination(form.next.as_deref());
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    let issued = service
        .verify_login_with_metadata(
            VerifyLogin {
                challenge_id: form.challenge_id,
                code: form.code,
            },
            &metadata,
        )
        .await?;
    form_response::redirect(
        &headers,
        &issued.raw_token,
        issued.metadata.idle_expires_at,
        destination,
    )
}
