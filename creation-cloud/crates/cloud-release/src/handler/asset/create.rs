//! 接收指定版本下的资产登记请求。

use axum::{Json, extract::Extension, extract::Path, extract::State, http::StatusCode};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{CreateAssetInput, ReleaseAsset, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
    Json(mut input): Json<CreateAssetInput>,
) -> AppResult<(StatusCode, Json<ReleaseAsset>)> {
    let actor = AdminActor::from_session(&session)?;
    input.release_id = release_id;
    Ok((
        StatusCode::CREATED,
        Json(service.create_asset(&actor, input).await?),
    ))
}
