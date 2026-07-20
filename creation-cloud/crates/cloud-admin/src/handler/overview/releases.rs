//! 返回版本状态只读概览。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{ReleaseOverview, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<ReleaseOverview>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.release_overview(&actor).await?))
}
