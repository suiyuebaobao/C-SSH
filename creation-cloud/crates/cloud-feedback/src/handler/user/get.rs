//! 按 URL 标识读取当前认证账号自己的反馈。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{FeedbackSubmission, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FeedbackSubmission>> {
    Ok(Json(service.get_own_feedback(&session, id).await?))
}
