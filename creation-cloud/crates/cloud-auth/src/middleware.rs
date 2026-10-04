//! 为受保护路由注入会话，并对状态变更请求强制校验 CSRF。

use axum::{
    Extension,
    body::{Body, to_bytes},
    extract::{OriginalUri, Request, State},
    http::{HeaderMap, Method, StatusCode, header::CONTENT_TYPE},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};

use crate::{Service, SessionMetadata, cookie, token};

const CSRF_HEADER: &str = "x-csrf-token";
const MAX_FORM_CSRF_BODY_BYTES: usize = 4 * 1024;

pub async fn require_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let raw_token = cookie::read(request.headers())?;
    let (session, metadata) = service.authenticate_with_metadata(&raw_token).await?;
    if requires_csrf(request.method()) {
        validate_csrf(request.headers(), &session.csrf_token)?;
    }
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(metadata.clone());
    let response = next.run(request).await;
    refresh_device_cookie(response, &raw_token, &metadata)
}

/// 只认证并注入 API 会话，供需要在 CSRF 之前审计的管理路由分层装配。
pub async fn authenticate_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let raw_token = cookie::read(request.headers())?;
    let (session, metadata) = service.authenticate_with_metadata(&raw_token).await?;
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(metadata.clone());
    let response = next.run(request).await;
    refresh_device_cookie(response, &raw_token, &metadata)
}

/// 页面 GET 在会话缺失时跳到登录页，其它方法仍返回可机读认证错误。
pub async fn require_page_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> Response {
    let (session, metadata, raw_token) =
        match authenticate_request(&service, request.headers()).await {
            Ok(session) => session,
            Err(_error) if request.method() == Method::GET || request.method() == Method::HEAD => {
                return login_redirect(&request).into_response();
            }
            Err(error) => return error.into_response(),
        };
    if requires_csrf(request.method()) {
        request = match validate_page_csrf(request, &session.csrf_token).await {
            Ok(request) => request,
            Err(response) => return response,
        };
    }
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(metadata.clone());
    let response = next.run(request).await;
    refresh_device_cookie(response, &raw_token, &metadata)
        .unwrap_or_else(IntoResponse::into_response)
}

/// 只认证并注入页面会话，状态变更请求的 CSRF 由内层管理中间件校验。
pub async fn authenticate_page_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> Response {
    let (session, metadata, raw_token) =
        match authenticate_request(&service, request.headers()).await {
            Ok(session) => session,
            Err(_error) if request.method() == Method::GET || request.method() == Method::HEAD => {
                return login_redirect(&request).into_response();
            }
            Err(error) => return error.into_response(),
        };
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    request.extensions_mut().insert(metadata.clone());
    let response = next.run(request).await;
    refresh_device_cookie(response, &raw_token, &metadata)
        .unwrap_or_else(IntoResponse::into_response)
}

/// 在管理审计层之内校验状态变更请求，使失败尝试也留下 failure 事件。
pub async fn require_csrf(
    Extension(session): Extension<AuthenticatedSession>,
    mut request: Request,
    next: Next,
) -> Response {
    if requires_csrf(request.method()) {
        request = match validate_page_csrf(request, &session.csrf_token).await {
            Ok(request) => request,
            Err(response) => return response,
        };
    }
    next.run(request).await
}

/// 拒绝普通账号进入管理端路由，身份必须先由 `require_session` 注入。
pub async fn require_admin(
    Extension(session): Extension<AuthenticatedSession>,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    if session.role != "admin" {
        return Err(AppError::Forbidden("需要管理员权限".to_owned()));
    }
    Ok(next.run(request).await)
}

fn requires_csrf(method: &Method) -> bool {
    !matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

fn validate_csrf(headers: &HeaderMap, expected: &str) -> AppResult<()> {
    let supplied = headers
        .get(CSRF_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("CSRF 校验失败".to_owned()))?;
    if !token::csrf_matches(expected, supplied) {
        return Err(AppError::Forbidden("CSRF 校验失败".to_owned()));
    }
    Ok(())
}

async fn validate_page_csrf(request: Request, expected: &str) -> Result<Request, Response> {
    if validate_csrf(request.headers(), expected).is_ok() {
        return Ok(request);
    }
    let is_form = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/x-www-form-urlencoded"));
    if !is_form {
        return Err(AppError::Forbidden("CSRF 校验失败".to_owned()).into_response());
    }
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, MAX_FORM_CSRF_BODY_BYTES)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE.into_response())?;
    let mut supplied = url::form_urlencoded::parse(&bytes)
        .filter(|(name, _)| name == "csrf_token")
        .map(|(_, value)| value);
    let valid = supplied
        .next()
        .is_some_and(|value| token::csrf_matches(expected, &value))
        && supplied.next().is_none();
    if !valid {
        return Err(AppError::Forbidden("CSRF 校验失败".to_owned()).into_response());
    }
    Ok(Request::from_parts(parts, Body::from(bytes)))
}

async fn authenticate_request(
    service: &Service,
    headers: &HeaderMap,
) -> AppResult<(AuthenticatedSession, SessionMetadata, String)> {
    let raw_token = cookie::read(headers)?;
    let (session, metadata) = service.authenticate_with_metadata(&raw_token).await?;
    Ok((session, metadata, raw_token))
}

fn refresh_device_cookie(
    mut response: Response,
    raw_token: &str,
    metadata: &SessionMetadata,
) -> AppResult<Response> {
    if metadata.session_kind == "device"
        && !response
            .headers()
            .contains_key(axum::http::header::SET_COOKIE)
    {
        response.headers_mut().insert(
            axum::http::header::SET_COOKIE,
            cookie::session_header(raw_token, metadata.idle_expires_at)?,
        );
    }
    Ok(response)
}

fn login_redirect(request: &Request) -> Redirect {
    let uri = request
        .extensions()
        .get::<OriginalUri>()
        .map_or(request.uri(), |original| &original.0);
    let next = uri
        .path_and_query()
        .map_or("/admin", axum::http::uri::PathAndQuery::as_str);
    let encoded = url::form_urlencoded::byte_serialize(next.as_bytes()).collect::<String>();
    Redirect::to(&format!("/login?next={encoded}"))
}
