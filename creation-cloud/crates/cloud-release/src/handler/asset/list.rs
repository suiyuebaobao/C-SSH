//! 返回某版本的全部资产身份。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseAsset, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
) -> AppResult<Json<Vec<ReleaseAsset>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_assets(&actor, release_id).await?))
}
