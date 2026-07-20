//! 接收管理员上传的本站资产文件并返回新来源。

use axum::{
    Json, extract::Extension, extract::Multipart, extract::Path, extract::State, http::StatusCode,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{ReleaseSource, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<ReleaseSource>)> {
    let actor = AdminActor::from_session(&session)?;
    Ok((
        StatusCode::CREATED,
        Json(service.upload_asset(&actor, asset_id, multipart).await?),
    ))
}
