//! 统一浏览器原生表单与 HTMX 表单的登录成功跳转语义。

use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use url::form_urlencoded::Serializer;
use uuid::Uuid;

use crate::captcha::CaptchaPurpose;
use crate::cookie;

const HX_REQUEST: &str = "hx-request";
const HX_REDIRECT: &str = "hx-redirect";

pub(crate) fn redirect(
    headers: &HeaderMap,
    raw_token: &str,
    expires_at: DateTime<Utc>,
    destination: &str,
) -> AppResult<Response> {
    let mut response = redirect_without_session(headers, destination)?;
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie::session_header(raw_token, expires_at)?,
    );
    Ok(response)
}

pub(crate) fn redirect_without_session(
    headers: &HeaderMap,
    destination: &str,
) -> AppResult<Response> {
    let htmx_request = headers
        .get(HX_REQUEST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));

    let mut response = if htmx_request {
        StatusCode::OK.into_response()
    } else {
        StatusCode::SEE_OTHER.into_response()
    };
    if htmx_request {
        response.headers_mut().insert(
            HeaderName::from_static(HX_REDIRECT),
            HeaderValue::from_str(destination)
                .map_err(|_| cloud_domain::AppError::Validation("登录返回地址无效".to_owned()))?,
        );
    } else {
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(destination)
                .map_err(|_| cloud_domain::AppError::Validation("登录返回地址无效".to_owned()))?,
        );
    }
    Ok(response)
}

pub(crate) fn safe_destination(value: Option<&str>) -> &str {
    let Some(value) = value else {
        return "/console";
    };
    let allowed_root = value == "/admin"
        || value.starts_with("/admin/")
        || value == "/console"
        || value.starts_with("/console/")
        || value == "/feedback"
        || value == "/en/feedback";
    if !allowed_root
        || value.len() > 256
        || value.starts_with("//")
        || value.contains(['\\', '#'])
        || value.chars().any(char::is_control)
    {
        "/console"
    } else {
        value
    }
}

pub(crate) fn captcha_purpose_for_destination(destination: &str) -> CaptchaPurpose {
    if destination == "/admin" || destination.starts_with("/admin/") {
        CaptchaPurpose::AdminLogin
    } else {
        CaptchaPurpose::Login
    }
}

pub(crate) fn login_verification_destination(
    is_en: bool,
    challenge_id: Uuid,
    next: Option<&str>,
    resent: bool,
) -> String {
    let mut query = Serializer::new(String::new());
    query.append_pair("challenge_id", &challenge_id.to_string());
    if let Some(next) = next {
        query.append_pair("next", next);
    }
    if resent {
        query.append_pair("resent", "1");
    }
    let path = if is_en {
        "/en/verify-login"
    } else {
        "/verify-login"
    };
    format!("{path}?{}", query.finish())
}
