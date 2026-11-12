//! 管理员账号设备限额的薄JSON入口，认证与CSRF由现有管理路由强制。
use crate::{AccountDevicePolicy, Service, UpdateDevicePolicy};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

pub(crate) async fn get(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account): Path<Uuid>,
) -> AppResult<Json<AccountDevicePolicy>> {
    Ok(Json(service.admin_device_policy(&session, account).await?))
}

pub(crate) async fn update(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account): Path<Uuid>,
    Json(input): Json<UpdateDevicePolicy>,
) -> AppResult<Json<AccountDevicePolicy>> {
    Ok(Json(
        service
            .admin_update_device_policy(&session, account, input)
            .await?,
    ))
}
