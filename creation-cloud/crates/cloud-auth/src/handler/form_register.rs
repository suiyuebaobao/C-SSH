//! 按服务端开关把注册表单送往邮箱验证页或直接建立会话。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;

use crate::{Register, RegistrationOutcome, Service};

use super::form_response;

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(mut command): Form<Register>,
) -> AppResult<Response> {
    if command.captcha_id.is_none() {
        command.captcha_id =
            crate::cookie::read_captcha(&headers, crate::captcha::CaptchaPurpose::Register);
    }
    let is_en = command.locale == "en";
    let verification_destination = if is_en {
        "/en/verify-email"
    } else {
        "/verify-email"
    };
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    let mut response = match service.register_with_metadata(command, &metadata).await? {
        RegistrationOutcome::VerificationRequired(_) => {
            form_response::redirect_without_session(&headers, verification_destination)
        }
        RegistrationOutcome::Session(issued) => form_response::redirect(
            &headers,
            &issued.raw_token,
            issued.metadata.idle_expires_at,
            "/console",
        ),
        RegistrationOutcome::Accepted(_) => form_response::redirect_without_session(
            &headers,
            if is_en { "/en/login" } else { "/login" },
        ),
    }?;
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        crate::cookie::clear_captcha_header(crate::captcha::CaptchaPurpose::Register)?,
    );
    Ok(response)
}
