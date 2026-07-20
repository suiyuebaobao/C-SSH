//! 接收尚未发布资产的身份补丁。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseAsset, Service, UpdateAssetInput};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    Json(input): Json<UpdateAssetInput>,
) -> AppResult<Json<ReleaseAsset>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update_asset(&actor, asset_id, input).await?))
}
