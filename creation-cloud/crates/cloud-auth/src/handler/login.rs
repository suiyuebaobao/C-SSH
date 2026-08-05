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
        let purpose = if command.identifier.trim().contains('@') {
            crate::captcha::CaptchaPurpose::Login
        } else {
            crate::captcha::CaptchaPurpose::AdminLogin
        };
        command.captcha_id = cookie::read_captcha(&headers, purpose);
    }
    let metadata = crate::TrustedRequestMetadata::from_headers(&headers);
    response(service.login_with_metadata(command, &metadata).await?)
}

fn response(outcome: LoginOutcome) -> AppResult<Response> {
    match outcome {
        LoginOutcome::VerificationRequired { status, is_admin } => {
            let purpose = if is_admin {
                crate::captcha::CaptchaPurpose::AdminLogin
            } else {
                crate::captcha::CaptchaPurpose::Login
            };
            let mut response = (StatusCode::ACCEPTED, Json(status)).into_response();
            response
                .headers_mut()
                .append(header::SET_COOKIE, cookie::clear_captcha_header(purpose)?);
            Ok(response)
        }
        LoginOutcome::Session(issued) => {
            let purpose = if issued.session.role == "admin" {
                crate::captcha::CaptchaPurpose::AdminLogin
            } else {
                crate::captcha::CaptchaPurpose::Login
            };
            let mut response = Json(issued.view()).into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                cookie::session_header(&issued.raw_token, issued.metadata.idle_expires_at)?,
            );
            response
                .headers_mut()
                .append(header::SET_COOKIE, cookie::clear_captcha_header(purpose)?);
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
    fn verification_required_clears_the_matching_captcha_without_a_session_cookie() {
        for (is_admin, expected_cookie) in [
            (false, "creation_captcha_login="),
            (true, "creation_captcha_admin="),
        ] {
            let response = response(LoginOutcome::VerificationRequired {
                status: LoginVerificationRequired {
                    status: "verification_required",
                    challenge_id: Uuid::now_v7(),
                    expires_at: Utc::now() + chrono::Duration::minutes(10),
                },
                is_admin,
            })
            .expect("challenge response");
            assert_eq!(response.status(), StatusCode::ACCEPTED);
            let cookies = response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .filter_map(|value| value.to_str().ok())
                .collect::<Vec<_>>();
            assert!(
                cookies
                    .iter()
                    .all(|value| !value.starts_with("creation_session="))
            );
            assert!(
                cookies
                    .iter()
                    .any(|value| value.starts_with(expected_cookie))
            );
        }
    }
}
