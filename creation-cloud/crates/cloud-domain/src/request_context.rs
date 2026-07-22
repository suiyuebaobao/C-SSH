//! 在单个异步 HTTP 请求内传播公开请求标识与语义审计完成状态。

use std::{cell::Cell, future::Future};

tokio::task_local! {
    static REQUEST_ID: String;
    static SEMANTIC_AUDIT_RECORDED: Cell<bool>;
}

pub async fn with_request_id<T>(request_id: String, future: impl Future<Output = T>) -> T {
    REQUEST_ID.scope(request_id, future).await
}

#[must_use]
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(Clone::clone).ok()
}

/// 在管理写请求内追踪业务责任层是否已提交语义审计，避免通用中间件重复记账。
pub async fn with_semantic_audit_tracking<T>(future: impl Future<Output = T>) -> (T, bool) {
    SEMANTIC_AUDIT_RECORDED
        .scope(Cell::new(false), async move {
            let output = future.await;
            let recorded = SEMANTIC_AUDIT_RECORDED.with(Cell::get);
            (output, recorded)
        })
        .await
}

/// 仅在业务写入与语义审计均已成功提交后调用；未处于管理请求作用域时安全地忽略。
pub fn mark_semantic_audit_recorded() {
    let _ = SEMANTIC_AUDIT_RECORDED.try_with(|recorded| recorded.set(true));
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

    #[tokio::test]
    async fn semantic_audit_tracking_is_request_scoped() {
        mark_semantic_audit_recorded();
        let (value, recorded) = with_semantic_audit_tracking(async {
            mark_semantic_audit_recorded();
            "完成"
        })
        .await;
        assert_eq!(value, "完成");
        assert!(recorded);

        let (_, next_recorded) = with_semantic_audit_tracking(async {}).await;
        assert!(!next_recorded);
    }
}
