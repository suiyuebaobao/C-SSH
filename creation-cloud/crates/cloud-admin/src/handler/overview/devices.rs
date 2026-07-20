//! 返回设备只读概览。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{DeviceOverview, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<DeviceOverview>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.device_overview(&actor).await?))
}
