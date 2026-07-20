//! 将设备登记 JSON 映射到当前认证账号。

use axum::{Extension, Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;

use crate::{CreateDevice, Device, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(command): Json<CreateDevice>,
) -> AppResult<(StatusCode, Json<Device>)> {
    let device = service.create(&session, command).await?;
    Ok((StatusCode::CREATED, Json(device)))
}
