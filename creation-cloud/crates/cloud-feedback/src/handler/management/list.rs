//! 从管理员会话派生 actor 后返回不含用户正文的最小摘要列表。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page};

use crate::{AdminFeedbackListQuery, AdminFeedbackSummary, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminFeedbackListQuery>,
) -> AppResult<Json<Page<AdminFeedbackSummary>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service.list_feedback_for_management(&actor, query).await?,
    ))
}
