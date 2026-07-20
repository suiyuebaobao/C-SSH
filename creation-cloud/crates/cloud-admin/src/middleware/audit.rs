//! 自动记录管理端非安全 HTTP 请求，只采集脱敏路径、状态和服务端会话身份。

use axum::{
    Extension,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, use_case::HttpAuditRecord};

const REQUEST_ID_HEADER: &str = "x-request-id";
const MAX_AUDIT_PATH_LEN: usize = 256;

#[derive(Clone, Copy, Debug)]
struct AuditRecorded;

pub async fn audit_write_requests(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    if is_safe(request.method()) {
        return Ok(next.run(request).await);
    }

    let actor = AdminActor::from_session(&session)?;
    let method = request.method().as_str().to_owned();
    let path = sanitize_path(request.uri().path());
    let (resource_kind, resource_id) = resource(request.uri().path());
    let request_id = request_id(request.headers());
    let audit_request_id = request_id.clone();
    let mut response = next.run(request).await;
    if response.extensions().get::<AuditRecorded>().is_some() {
        return Ok(response);
    }
    service
        .record_http_request(
            &actor,
            HttpAuditRecord {
                method,
                path,
                resource_kind,
                resource_id,
                status: response.status().as_u16(),
                request_id: audit_request_id,
            },
        )
        .await?;
    let response_request_id = HeaderValue::from_str(&request_id)
        .map_err(|_| AppError::Internal("请求标识无法写入响应".to_owned()))?;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, response_request_id);
    response.extensions_mut().insert(AuditRecorded);
    Ok(response)
}

fn is_safe(method: &Method) -> bool {
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

pub(crate) fn request_id(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
                })
        })
        .map_or_else(|| Uuid::now_v7().to_string(), str::to_owned)
}

pub(crate) fn sanitize_path(path: &str) -> String {
    let mut output = String::with_capacity(path.len().min(MAX_AUDIT_PATH_LEN));
    output.push('/');
    for (index, segment) in path.trim_matches('/').split('/').enumerate() {
        if index > 0 {
            output.push('/');
        }
        let safe = is_known_segment(segment) || Uuid::parse_str(segment).is_ok();
        let value = if safe { segment } else { "redacted" };
        let remaining = MAX_AUDIT_PATH_LEN.saturating_sub(output.len());
        if remaining == 0 {
            break;
        }
        output.extend(value.chars().take(remaining));
    }
    output
}

fn is_known_segment(segment: &str) -> bool {
    matches!(
        segment,
        "api"
            | "v1"
            | "admin"
            | "overview"
            | "users"
            | "devices"
            | "releases"
            | "assets"
            | "downloads"
            | "sources"
            | "site-media"
            | "feedback"
            | "status"
            | "redact"
            | "publish"
            | "revoke"
            | "audit-events"
    )
}

fn resource(path: &str) -> (String, Option<String>) {
    let segments = path.trim_matches('/').split('/').collect::<Vec<_>>();
    let start = segments
        .iter()
        .position(|segment| *segment == "admin")
        .map_or(0, |index| index + 1);
    let kind = segments
        .get(start)
        .copied()
        .filter(|segment| is_known_segment(segment))
        .unwrap_or("admin")
        .to_owned();
    let id = segments
        .iter()
        .skip(start + 1)
        .find_map(|segment| Uuid::parse_str(segment).ok())
        .map(|value| value.to_string());
    (kind, id)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderValue, Uri};

    use super::*;

    #[test]
    fn strips_query_and_unknown_path_values() {
        let uri: Uri = "/api/v1/admin/users/admin@example.com?token=secret"
            .parse()
            .expect("测试 URI 应有效");
        let value = sanitize_path(uri.path());
        assert_eq!(value, "/api/v1/admin/users/redacted");
        assert!(!value.contains("example"));
        assert!(!value.contains("token"));
        assert!(!value.contains('?'));
    }

    #[test]
    fn rejects_unsafe_request_id_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_static("request@example.com"),
        );
        let generated = request_id(&headers);
        assert!(Uuid::parse_str(&generated).is_ok());
    }

    #[test]
    fn keeps_feedback_action_segments_and_extracts_feedback_identity() {
        let feedback_id = Uuid::now_v7();
        let path = format!("/api/v1/admin/feedback/{feedback_id}/redact");
        assert_eq!(sanitize_path(&path), path);
        assert_eq!(
            resource(&path),
            ("feedback".to_owned(), Some(feedback_id.to_string()))
        );
    }
}
