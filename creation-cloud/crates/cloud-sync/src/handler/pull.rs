//! 把已认证账号的增量 pull 查询映射到 pull 用例。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{PullRequest, PullResponse, Service};

pub(crate) async fn pull(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Query(request): Query<PullRequest>,
) -> AppResult<Json<PullResponse>> {
    service.pull(account_id, request).await.map(Json)
}
