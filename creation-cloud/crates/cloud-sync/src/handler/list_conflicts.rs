//! 把已认证账号的冲突列表查询映射到列表用例。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{Service, SyncConflict};

pub(crate) async fn list_conflicts(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<SyncConflict>>> {
    service.list_conflicts(&session, page).await.map(Json)
}
