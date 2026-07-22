//! 撤销本人设备并复用设备域的会话原子清理语义。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct RevokeDeviceForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<RevokeDeviceForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state.device().delete(&session, device_id).await?;
    Ok(common::action_success(&headers, PageId::Devices, locale))
}
