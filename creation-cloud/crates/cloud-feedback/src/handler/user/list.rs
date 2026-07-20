//! 解析受限分页参数并返回当前认证账号自己的反馈列表。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{FeedbackSubmission, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<FeedbackSubmission>>> {
    Ok(Json(service.list_own_feedback(&session, page).await?))
}
