//! 将代理控制面的失败映射为固定状态码和不含目标或秘密的错误正文。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cloud_domain::current_request_id;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("官方代理请求无效")]
    InvalidRequest,
    #[error("当前会话尚未绑定活动设备")]
    DeviceRequired,
    #[error("当前账号或设备会话状态已变化")]
    SessionChanged,
    #[error("官方代理当前不可用")]
    Unavailable,
    #[error("mutation 已绑定到不同请求")]
    MutationConflict,
    #[error("官方代理签发频率或活动凭据数量已达上限")]
    RateLimited,
    #[error("官方代理凭据不存在")]
    CredentialNotFound,
    #[error("代理鉴权失败")]
    AuthorizationDenied,
    #[error("官方代理内部错误")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message_key: &'static str,
    message: &'static str,
    request_id: String,
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest => (
                StatusCode::BAD_REQUEST,
                "proxy_invalid_request",
                "官方代理请求无效",
            ),
            Self::DeviceRequired => (
                StatusCode::CONFLICT,
                "proxy_device_required",
                "当前会话尚未绑定活动设备",
            ),
            Self::SessionChanged => (
                StatusCode::CONFLICT,
                "proxy_session_changed",
                "当前账号或设备会话状态已变化",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "official_proxy_unavailable",
                "官方代理当前不可用",
            ),
            Self::MutationConflict => (
                StatusCode::CONFLICT,
                "proxy_mutation_conflict",
                "mutation 已绑定到不同请求",
            ),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "proxy_rate_limited",
                "官方代理签发频率或活动凭据数量已达上限",
            ),
            Self::CredentialNotFound => (
                StatusCode::NOT_FOUND,
                "proxy_credential_not_found",
                "官方代理凭据不存在",
            ),
            Self::AuthorizationDenied => (
                StatusCode::UNAUTHORIZED,
                "proxy_authorization_denied",
                "代理鉴权失败",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "官方代理内部错误",
            ),
        };
        (
            status,
            Json(ErrorBody {
                code,
                message_key: code,
                message,
                request_id: current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string()),
            }),
        )
            .into_response()
    }
}

pub(crate) fn storage(_error: sqlx::Error) -> ProxyError {
    // SQL 细节可能包含受限元数据，固定折叠且不写日志。
    ProxyError::Internal
}
