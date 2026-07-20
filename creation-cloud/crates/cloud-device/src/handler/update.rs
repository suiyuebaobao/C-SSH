//! 从路径与 JSON 提取设备重命名请求并调用更新用例。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::{Device, Service, UpdateDevice};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    Json(command): Json<UpdateDevice>,
) -> AppResult<Json<Device>> {
    Ok(Json(service.update(&session, device_id, command).await?))
}
