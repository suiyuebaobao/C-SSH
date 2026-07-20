//! 从管理员会话派生 actor 后显式读取单条完整纯文本详情。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminFeedbackDetail, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AdminFeedbackDetail>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_feedback_for_management(&actor, id).await?))
}
