//! 仅从 AuthenticatedSession 获取身份并映射包装密钥分页请求。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{Service, VaultKeyWrapper};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<VaultKeyWrapper>>> {
    service.list_wrappers(&session, page).await.map(Json)
}
