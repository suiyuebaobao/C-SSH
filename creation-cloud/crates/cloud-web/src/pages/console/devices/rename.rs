//! 重命名本人未撤销设备，不改变设备公开标识或会话绑定。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_device::UpdateDevice;
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct RenameDeviceForm {
    name: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<RenameDeviceForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state
        .device()
        .update(&session, device_id, UpdateDevice { name: form.name })
        .await?;
    Ok(common::action_success(&headers, PageId::Devices, locale))
}
