//! 返回单个下载来源的管理视图。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseSource, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(source_id): Path<Uuid>,
) -> AppResult<Json<ReleaseSource>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_source(&actor, source_id).await?))
}
