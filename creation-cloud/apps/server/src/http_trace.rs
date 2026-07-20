//! 为 HTTP 请求生成不含查询参数和请求头的结构化追踪跨度。
//! 邮箱筛选、下载签名与会话字段不得因默认 URI 记录而进入日志。

use axum::{body::Body, http::Request};
use tracing::Span;

pub(crate) fn make_span(request: &Request<Body>) -> Span {
    tracing::info_span!(
        "http.request",
        method = %request.method(),
        path = %request.uri().path()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_path_excludes_sensitive_query_values() {
        let request = Request::builder()
            .uri("/api/v1/admin/users?email=private%40example.com&token=secret")
            .body(Body::empty())
            .expect("测试请求应可创建");
        assert_eq!(request.uri().path(), "/api/v1/admin/users");
    }
}
