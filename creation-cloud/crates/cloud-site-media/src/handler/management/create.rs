//! 接收管理员上传并创建首页二维码草稿。

use axum::{
    Extension,
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{Service, SiteMedia, multipart};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<SiteMedia>)> {
    let actor = AdminActor::from_session(&session)?;
    let input = multipart::parse(multipart).await?;
    Ok((
        StatusCode::CREATED,
        Json(service.create(&actor, input).await?),
    ))
}
