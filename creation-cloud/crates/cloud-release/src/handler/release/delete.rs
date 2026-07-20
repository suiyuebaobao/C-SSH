//! 删除仍为草稿的版本并返回空响应。

use axum::{extract::Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_release(&actor, release_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
