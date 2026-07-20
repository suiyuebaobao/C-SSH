//! 统一定义可跨业务模块传播并映射为 HTTP 的错误。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::current_request_id;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    RateLimited(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Storage(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message_key: &'static str,
    message: String,
    request_id: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Self::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            Self::Unavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            Self::Storage(_) | Self::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        let body = ErrorBody {
            code,
            message_key: code,
            message: self.to_string(),
            request_id: current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string()),
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;
    use crate::with_request_id;

    #[tokio::test]
    async fn error_body_carries_stable_key_and_scoped_request_id() {
        let response = with_request_id("request-example".to_owned(), async {
            AppError::Validation("输入无效".to_owned()).into_response()
        })
        .await;
        let body = to_bytes(response.into_body(), 4096)
            .await
            .expect("错误正文应可读取");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("错误正文应为 JSON");
        assert_eq!(value["code"], "validation_error");
        assert_eq!(value["message_key"], "validation_error");
        assert_eq!(value["request_id"], "request-example");
    }

    #[tokio::test]
    async fn unavailable_error_has_a_stable_service_unavailable_boundary() {
        let response = AppError::Unavailable("请稍后重试".to_owned()).into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(response.into_body(), 4096)
            .await
            .expect("错误正文应可读取");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("正文应为 JSON");
        assert_eq!(value["code"], "unavailable");
        assert_eq!(value["message_key"], "unavailable");
    }

    #[tokio::test]
    async fn rate_limited_error_has_a_stable_too_many_requests_boundary() {
        let response = AppError::RateLimited("提交过于频繁".to_owned()).into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = to_bytes(response.into_body(), 4096)
            .await
            .expect("错误正文应可读取");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("正文应为 JSON");
        assert_eq!(value["code"], "rate_limited");
        assert_eq!(value["message_key"], "rate_limited");
    }
}
