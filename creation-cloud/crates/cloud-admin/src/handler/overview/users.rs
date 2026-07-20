//! 返回用户只读概览。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{Service, UserOverview};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<UserOverview>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.user_overview(&actor).await?))
}
