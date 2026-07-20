//! 在单个异步 HTTP 请求内传播公开请求标识，供统一错误与审计关联。

use std::future::Future;

tokio::task_local! {
    static REQUEST_ID: String;
}

pub async fn with_request_id<T>(request_id: String, future: impl Future<Output = T>) -> T {
    REQUEST_ID.scope(request_id, future).await
}

#[must_use]
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(Clone::clone).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_id_is_visible_only_inside_its_async_scope() {
        assert!(current_request_id().is_none());
        let value = with_request_id("request-example".to_owned(), async {
            current_request_id().expect("作用域内应存在请求标识")
        })
        .await;
        assert_eq!(value, "request-example");
        assert!(current_request_id().is_none());
    }
}
