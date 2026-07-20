//! 解析分页参数并返回当前账号的资料列表。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppResult, Page, PageQuery};

use crate::{Profile, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<Profile>>> {
    Ok(Json(service.list(&session, page).await?))
}
