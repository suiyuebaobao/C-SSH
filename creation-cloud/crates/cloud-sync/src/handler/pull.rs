//! 把已认证账号的增量 pull 查询映射到 pull 用例。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{PullRequest, PullResponse, Service};

pub(crate) async fn pull(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(request): Query<PullRequest>,
) -> AppResult<Json<PullResponse>> {
    service.pull(&session, request).await.map(Json)
}
