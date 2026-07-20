//! 解析分页参数并返回当前账号的设备列表。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppResult, Page, PageQuery};

use crate::{Device, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<Device>>> {
    Ok(Json(service.list(&session, page).await?))
}
