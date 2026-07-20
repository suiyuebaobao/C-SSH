//! 返回单个资产身份，不混入下载来源。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseAsset, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
) -> AppResult<Json<ReleaseAsset>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_asset(&actor, asset_id).await?))
}
