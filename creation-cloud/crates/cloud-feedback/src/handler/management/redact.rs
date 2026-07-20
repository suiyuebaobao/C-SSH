//! 从管理员会话派生 actor 后执行有原因校验的不可逆安全脱敏。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminFeedbackDetail, RedactFeedbackInput, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<RedactFeedbackInput>,
) -> AppResult<Json<AdminFeedbackDetail>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.redact_feedback(&actor, id, input).await?))
}
