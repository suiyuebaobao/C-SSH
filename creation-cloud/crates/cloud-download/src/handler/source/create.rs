//! 接收资产下的新下载来源。

use axum::{Json, extract::Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{CreateSourceInput, ReleaseSource, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    Json(mut input): Json<CreateSourceInput>,
) -> AppResult<(StatusCode, Json<ReleaseSource>)> {
    let actor = AdminActor::from_session(&session)?;
    input.asset_id = asset_id;
    Ok((
        StatusCode::CREATED,
        Json(service.create_source(&actor, input).await?),
    ))
}
