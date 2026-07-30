//! 按服务端开关把注册表单送往邮箱验证页或直接建立会话。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;

use crate::{Register, RegistrationOutcome, Service};

use super::form_response;

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(command): Form<Register>,
) -> AppResult<Response> {
    let is_en = command.locale == "en";
    let verification_destination = if is_en {
        "/en/verify-email"
    } else {
        "/verify-email"
    };
    match service.register(command).await? {
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
    }
}
