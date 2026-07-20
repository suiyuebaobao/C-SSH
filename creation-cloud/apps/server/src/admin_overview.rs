//! 在服务装配层组合管理统计与基础设施健康状态。
//! 这里只调用公开用例和就绪探针，不跨领域直接查询业务表。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use serde::Serialize;

#[derive(Clone)]
pub(crate) struct AdminOverviewState {
    admin: cloud_admin::Service,
    feedback: cloud_feedback::Service,
    page: cloud_web::AdminPageState,
}

impl AdminOverviewState {
    pub(crate) const fn new(
        admin: cloud_admin::Service,
        feedback: cloud_feedback::Service,
        page: cloud_web::AdminPageState,
    ) -> Self {
        Self {
            admin,
            feedback,
            page,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct AdminOverviewResponse {
    #[serde(flatten)]
    overview: cloud_admin::AdminOverview,
    feedback: cloud_feedback::FeedbackOverview,
    health: cloud_web::AdminHealth,
}

pub(crate) async fn handle(
    State(state): State<AdminOverviewState>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<AdminOverviewResponse>> {
    let actor = AdminActor::from_session(&session)?;
    let (overview, feedback, health) = tokio::join!(
        state.admin.overview(&actor),
        state.feedback.overview(&actor),
        state.page.health()
    );
    Ok(Json(AdminOverviewResponse {
        overview: overview?,
        feedback: feedback?,
        health,
    }))
}
