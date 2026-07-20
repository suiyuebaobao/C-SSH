//! 接收来源排序或启停补丁。

use axum::{Json, extract::Extension, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseSource, Service, UpdateSourceInput};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(source_id): Path<Uuid>,
    Json(input): Json<UpdateSourceInput>,
) -> AppResult<Json<ReleaseSource>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update_source(&actor, source_id, input).await?))
}
