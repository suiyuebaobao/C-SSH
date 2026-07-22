//! 把已认证账号的同步 push 请求映射到 push 用例。

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{PushOutcome, PushRequest, Service};

pub(crate) async fn push(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(request): Json<PushRequest>,
) -> AppResult<Response> {
    let outcome = service.push(&session, request).await?;
    let status = match outcome {
        PushOutcome::Applied { .. } => StatusCode::OK,
        PushOutcome::Conflict { .. } => StatusCode::CONFLICT,
    };
    Ok((status, Json(outcome)).into_response())
}
