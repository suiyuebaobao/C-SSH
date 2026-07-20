//! 接收版本元数据补丁或状态迁移请求。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Release, Service, UpdateReleaseInput};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
    Json(input): Json<UpdateReleaseInput>,
) -> AppResult<Json<Release>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(
        service.update_release(&actor, release_id, input).await?,
    ))
}
