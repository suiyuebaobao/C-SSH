//! 接收登录 JSON，写入安全会话 Cookie 并返回会话视图。

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;

use crate::{Login, LoginOutcome, Service, cookie};

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(mut command): Json<Login>,
) -> AppResult<Response> {
    if command.captcha_id.is_none() {
        command.captcha_id = cookie::read_admin_captcha(&headers);
    }
    response(service.login(command).await?)
}

fn response(outcome: LoginOutcome) -> AppResult<Response> {
    match outcome {
        LoginOutcome::VerificationRequired(status) => {
            Ok((StatusCode::ACCEPTED, Json(status)).into_response())
        }
        LoginOutcome::Session(issued) => {
            let mut response = Json(issued.view()).into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
            );
            response
                .headers_mut()
                .append(header::SET_COOKIE, cookie::clear_admin_captcha_header()?);
            Ok(response)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::LoginVerificationRequired;

    #[test]
    fn verification_required_never_sets_a_session_cookie() {
        let response = response(LoginOutcome::VerificationRequired(
            LoginVerificationRequired {
                status: "verification_required",
                challenge_id: Uuid::now_v7(),
                expires_at: Utc::now() + chrono::Duration::minutes(10),
            },
        ))
        .expect("challenge response");
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }
}
