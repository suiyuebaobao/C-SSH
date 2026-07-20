//! 把已认证账号的同步 push 请求映射到 push 用例。

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{PushOutcome, PushRequest, Service};

pub(crate) async fn push(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Json(request): Json<PushRequest>,
) -> AppResult<Response> {
    let outcome = service.push(account_id, request).await?;
    let status = match outcome {
        PushOutcome::Applied { .. } => StatusCode::OK,
        PushOutcome::Conflict { .. } => StatusCode::CONFLICT,
    };
    Ok((status, Json(outcome)).into_response())
}
