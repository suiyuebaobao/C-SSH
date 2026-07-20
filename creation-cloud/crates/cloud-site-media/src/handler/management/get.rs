//! 返回管理员可见的单条站点媒体元数据。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, SiteMedia};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
) -> AppResult<Json<SiteMedia>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get(&actor, media_id).await?))
}
