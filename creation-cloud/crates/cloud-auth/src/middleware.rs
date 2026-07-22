//! 为受保护路由注入会话，并对状态变更请求强制校验 CSRF。

use axum::{
    Extension,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri, header::CONTENT_TYPE},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};

use crate::{Service, cookie, token};

const CSRF_HEADER: &str = "x-csrf-token";
const MAX_FORM_CSRF_BODY_BYTES: usize = 4 * 1024;

pub async fn require_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let raw_token = cookie::read(request.headers())?;
    let session = service.authenticate(&raw_token).await?;
    if requires_csrf(request.method()) {
        validate_csrf(request.headers(), &session.csrf_token)?;
    }
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    Ok(next.run(request).await)
}

/// 只认证并注入 API 会话，供需要在 CSRF 之前审计的管理路由分层装配。
pub async fn authenticate_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let raw_token = cookie::read(request.headers())?;
    let session = service.authenticate(&raw_token).await?;
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    Ok(next.run(request).await)
}

/// 页面 GET 在会话缺失时跳到登录页，其它方法仍返回可机读认证错误。
pub async fn require_page_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> Response {
    let session = match authenticate_request(&service, request.headers()).await {
        Ok(session) => session,
        Err(_error) if request.method() == Method::GET || request.method() == Method::HEAD => {
            return login_redirect(request.uri()).into_response();
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
    next.run(request).await
}

/// 只认证并注入页面会话，状态变更请求的 CSRF 由内层管理中间件校验。
pub async fn authenticate_page_session(
    State(service): State<Service>,
    mut request: Request,
    next: Next,
) -> Response {
    let session = match authenticate_request(&service, request.headers()).await {
        Ok(session) => session,
        Err(_error) if request.method() == Method::GET || request.method() == Method::HEAD => {
            return login_redirect(request.uri()).into_response();
        }
        Err(error) => return error.into_response(),
    };
    request.extensions_mut().insert(session.account_id);
    request.extensions_mut().insert(session);
    next.run(request).await
}

/// 在管理审计层之内校验状态变更请求，使失败尝试也留下 failure 事件。
pub async fn require_csrf(
    Extension(session): Extension<AuthenticatedSession>,
    request: Request,
    next: Next,
) -> AppResult<Response> {
    if requires_csrf(request.method()) {
        validate_csrf(request.headers(), &session.csrf_token)?;
    }
    Ok(next.run(request).await)
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
) -> AppResult<AuthenticatedSession> {
    let raw_token = cookie::read(headers)?;
    service.authenticate(&raw_token).await
}

fn login_redirect(uri: &Uri) -> Redirect {
    let next = uri
        .path_and_query()
        .map_or("/admin", axum::http::uri::PathAndQuery::as_str);
    let encoded = url::form_urlencoded::byte_serialize(next.as_bytes()).collect::<String>();
    Redirect::to(&format!("/login?next={encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_redirect_keeps_only_the_local_request_target() {
        let uri: Uri = "/admin/releases?lang=en".parse().expect("URI 应有效");
        let response = login_redirect(&uri).into_response();
        assert_eq!(
            response.headers()[axum::http::header::LOCATION],
            "/login?next=%2Fadmin%2Freleases%3Flang%3Den"
        );
    }

    #[tokio::test]
    async fn native_form_csrf_is_validated_and_body_is_restored() {
        let request = Request::builder()
            .method(Method::POST)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("csrf_token=csrf-example&display_name=Example"))
            .expect("表单请求应可构造");
        let request = validate_page_csrf(request, "csrf-example")
            .await
            .expect("隐藏字段中的有效 CSRF 应被接受");
        let body = to_bytes(request.into_body(), MAX_FORM_CSRF_BODY_BYTES)
            .await
            .expect("校验后表单正文仍应可读");
        assert_eq!(body, "csrf_token=csrf-example&display_name=Example");
    }

    #[tokio::test]
    async fn native_form_csrf_rejects_duplicates_and_oversized_bodies() {
        let duplicate = Request::builder()
            .method(Method::POST)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(
                "csrf_token=csrf-example&csrf_token=csrf-example",
            ))
            .expect("重复令牌请求应可构造");
        assert_eq!(
            validate_page_csrf(duplicate, "csrf-example")
                .await
                .expect_err("重复令牌必须拒绝")
                .status(),
            StatusCode::FORBIDDEN
        );

        let oversized = Request::builder()
            .method(Method::POST)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(vec![b'a'; MAX_FORM_CSRF_BODY_BYTES + 1]))
            .expect("超限请求应可构造");
        assert_eq!(
            validate_page_csrf(oversized, "csrf-example")
                .await
                .expect_err("超限表单必须在解析前拒绝")
                .status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }
}
