//! 当前设备身份补登记的薄HTTP入口。
use crate::{BindDeviceIdentity, Device, Service};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

pub(crate) async fn bind(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device): Path<Uuid>,
    Json(input): Json<BindDeviceIdentity>,
) -> AppResult<Json<Device>> {
    Ok(Json(service.bind_identity(&session, device, input).await?))
}
