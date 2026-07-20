//! 返回指定版本的管理视图。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Release, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
) -> AppResult<Json<Release>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_release(&actor, release_id).await?))
}
