//! 从路径读取设备 ID 并返回当前账号拥有的设备。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::{Device, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
) -> AppResult<Json<Device>> {
    Ok(Json(service.get(&session, device_id).await?))
}
