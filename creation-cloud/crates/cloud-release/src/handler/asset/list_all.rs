//! 返回跨版本的分页资产管理清单。

use axum::{Json, extract::Extension, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{ReleaseAsset, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Page<ReleaseAsset>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_all_assets(&actor, query).await?))
}
