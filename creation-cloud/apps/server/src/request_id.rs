//! 为每个 HTTP 请求生成或收敛可公开关联的请求标识，并回写响应头。

use axum::{
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub async fn attach(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid(value))
        .map_or_else(|| Uuid::now_v7().to_string(), str::to_owned);
    let header_value = HeaderValue::from_str(&request_id)
        .unwrap_or_else(|_| HeaderValue::from_static("request-id-invalid"));
    request
        .headers_mut()
        .insert(REQUEST_ID.clone(), header_value.clone());
    let mut response = cloud_domain::with_request_id(request_id, next.run(request)).await;
    response.headers_mut().insert(REQUEST_ID, header_value);
    response
}

fn valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::valid;

    #[test]
    fn request_id_rejects_sensitive_or_header_breaking_characters() {
        assert!(valid("019f-example:01"));
        assert!(!valid("admin@example.com"));
        assert!(!valid("line\nbreak"));
        assert!(!valid(""));
    }
}
