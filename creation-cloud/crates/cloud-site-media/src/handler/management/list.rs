//! 返回管理员可见的首页二维码历史列表。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use serde::Deserialize;

use crate::{Service, SiteMedia};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListQuery {
    limit: Option<u32>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<SiteMedia>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list(&actor, query.limit).await?))
}
