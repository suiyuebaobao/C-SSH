//! 删除仍处于草稿状态的站点媒体及其受控文件。

use axum::{Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete(&actor, media_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
