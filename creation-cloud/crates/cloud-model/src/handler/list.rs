//! 把已认证账号的模型分页查询映射到 list 用例。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, Page, PageQuery};
use uuid::Uuid;

use crate::{ModelProfile, Service};

pub(crate) async fn list(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<ModelProfile>>> {
    service.list(account_id, page).await.map(Json)
}
