//! 删除尚未发布且没有来源引用的资产。

use axum::{extract::Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::Service;

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
    service.delete_asset(&actor, asset_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
