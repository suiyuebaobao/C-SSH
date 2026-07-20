//! 从服务端会话派生管理员身份并撤销设备授权。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AdminDevice, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
) -> AppResult<Json<AdminDevice>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.revoke_device(&actor, device_id).await?))
}
