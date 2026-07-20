//! 接收管理员对二维码草稿双语替代文本的更新。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, SiteMedia, UpdateSiteMediaInput};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
    Json(input): Json<UpdateSiteMediaInput>,
) -> AppResult<Json<SiteMedia>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.update(&actor, media_id, input).await?))
}
