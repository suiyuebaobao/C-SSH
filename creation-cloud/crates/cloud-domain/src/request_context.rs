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
