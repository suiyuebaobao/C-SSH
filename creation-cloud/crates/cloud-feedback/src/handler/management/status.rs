//! 从管理员会话派生 actor 后执行带期望版本的受控状态迁移。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminFeedbackDetail, Service, UpdateFeedbackStatusInput};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFeedbackStatusInput>,
) -> AppResult<Json<AdminFeedbackDetail>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service.update_feedback_status(&actor, id, input).await?,
    ))
}
