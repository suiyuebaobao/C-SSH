//! 将受限反馈 JSON 映射到当前认证账号，成功入库后才返回真实 UUID。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{CreateFeedbackInput, FeedbackSubmission, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateFeedbackInput>,
) -> AppResult<(StatusCode, Json<FeedbackSubmission>)> {
    let feedback = service.create_feedback(&session, input).await?;
    Ok((StatusCode::CREATED, Json(feedback)))
}
