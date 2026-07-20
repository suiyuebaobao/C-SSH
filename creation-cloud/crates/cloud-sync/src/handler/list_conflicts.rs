//! 把已认证账号的冲突列表查询映射到列表用例。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, Page, PageQuery};
use uuid::Uuid;

use crate::{Service, SyncConflict};

pub(crate) async fn list_conflicts(
    State(service): State<Service>,
    Extension(account_id): Extension<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<SyncConflict>>> {
    service.list_conflicts(account_id, page).await.map(Json)
}
