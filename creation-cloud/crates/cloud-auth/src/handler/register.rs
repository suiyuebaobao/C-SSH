//! 按服务端开关创建待验证账号或直接签发安全会话。

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Register, RegistrationOutcome, Service, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<Register>,
) -> AppResult<Response> {
    response(service.register(command).await?)
}

fn response(outcome: RegistrationOutcome) -> AppResult<Response> {
    match outcome {
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
    }
}
