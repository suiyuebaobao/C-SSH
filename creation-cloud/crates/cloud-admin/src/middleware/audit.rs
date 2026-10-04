//! 自动记录管理端非安全 HTTP 请求，只采集脱敏路径、状态和服务端会话身份。

use axum::{
    Extension,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use cloud_domain::{
    AdminActor, AppError, AppResult, AuthenticatedSession, with_semantic_audit_tracking,
};
use uuid::Uuid;

use crate::{Service, use_case::HttpAuditRecord};

const REQUEST_ID_HEADER: &str = "x-request-id";
const MAX_AUDIT_PATH_LEN: usize = 256;

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
    let (mut response, semantic_audit_recorded) =
        with_semantic_audit_tracking(next.run(request)).await;
    let response_request_id = HeaderValue::from_str(&request_id)
        .map_err(|_| AppError::Internal("请求标识无法写入响应".to_owned()))?;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, response_request_id);
    if semantic_audit_recorded {
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
            | "announcements"
            | "releases"
            | "assets"
            | "downloads"
            | "sources"
            | "site-media"
            | "feedback"
            | "seo"
            | "status"
            | "redact"
            | "hide"
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
