//! 返回分页版本管理清单。

use axum::{Json, extract::Extension, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{Release, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Page<Release>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_releases(&actor, query).await?))
}
