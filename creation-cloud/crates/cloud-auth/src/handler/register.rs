//! 按服务端开关创建待验证账号或直接签发安全会话。

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Register, RegistrationOutcome, Service, captcha::CaptchaPurpose, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(mut command): Json<Register>,
) -> AppResult<Response> {
    if command.captcha_id.is_none() {
        command.captcha_id = cookie::read_captcha(&headers, CaptchaPurpose::Register);
    }
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    response(service.register_with_metadata(command, &metadata).await?)
}

fn response(outcome: RegistrationOutcome) -> AppResult<Response> {
    let mut response = match outcome {
        RegistrationOutcome::VerificationRequired(status)
        | RegistrationOutcome::Accepted(status) => {
            Ok((StatusCode::ACCEPTED, Json(status)).into_response())
        }
        RegistrationOutcome::Session(issued) => {
            let mut response = (StatusCode::CREATED, Json(issued.view())).into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
            );
            Ok(response)
        }
    }?;
    response.headers_mut().append(
        header::SET_COOKIE,
        cookie::clear_captcha_header(CaptchaPurpose::Register)?,
    );
    Ok(response)
}
