//! 统一浏览器原生表单与 HTMX 表单的登录成功跳转语义。

use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use url::form_urlencoded::Serializer;
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htmx_uses_client_redirect_without_redirect_body_swap() {
        let mut headers = HeaderMap::new();
        headers.insert(HX_REQUEST, HeaderValue::from_static("true"));

        let response = redirect(
            &headers,
            "token",
            Utc::now() + chrono::Duration::hours(1),
            "/admin/releases",
        )
        .expect("build response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[HX_REDIRECT], "/admin/releases");
        assert!(!response.headers().contains_key(header::LOCATION));
    }

    #[test]
    fn native_form_uses_see_other_location() {
        let response = redirect(
            &HeaderMap::new(),
            "token",
            Utc::now() + chrono::Duration::hours(1),
            "/console",
        )
        .expect("build response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()[header::LOCATION], "/console");
        assert!(!response.headers().contains_key(HX_REDIRECT));
    }

    #[test]
    fn unsafe_destination_falls_back_to_console() {
        assert_eq!(safe_destination(Some("https://example.com")), "/console");
        assert_eq!(safe_destination(Some("//example.com/admin")), "/console");
        assert_eq!(
            safe_destination(Some("/admin/audit?lang=en")),
            "/admin/audit?lang=en"
        );
        assert_eq!(safe_destination(Some("/feedback")), "/feedback");
    }

    #[test]
    fn verification_destination_preserves_language_and_safe_next() {
        let challenge_id = Uuid::now_v7();
        let destination = login_verification_destination(
            true,
            challenge_id,
            Some("/admin/releases?lang=en"),
            true,
        );
        assert!(destination.starts_with("/en/verify-login?challenge_id="));
        assert!(destination.contains("next=%2Fadmin%2Freleases%3Flang%3Den"));
        assert!(destination.ends_with("resent=1"));
    }
}
