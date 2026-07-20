//! 返回资产的全部来源并保留管理排序。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseSource, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
) -> AppResult<Json<Vec<ReleaseSource>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_sources(&actor, asset_id).await?))
}
