//! 为公网代理 API 的成功和鉴权失败响应统一附加禁止缓存头。

use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

pub async fn attach(request: Request, next: Next) -> Response {
    let is_proxy_api = request
        .uri()
        .path()
        .strip_prefix("/api/v1/proxy")
        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'));
    let mut response = next.run(request).await;
    if is_proxy_api {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
            .headers_mut()
            .insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    }
    response
}
