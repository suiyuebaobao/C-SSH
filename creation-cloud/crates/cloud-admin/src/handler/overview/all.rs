//! 返回管理端四域组合概览。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{AdminOverview, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<AdminOverview>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.overview(&actor).await?))
}
