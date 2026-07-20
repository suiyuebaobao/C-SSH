//! 处理管理员撤销单个客户端设备的请求。
//! 该动作只软撤销设备元数据，不删除账号、同步数据或任何 SSH 资料。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct RevokeDeviceForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<RevokeDeviceForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.admin().revoke_device(&actor, device_id).await {
        Ok(_) => shared::action_success(&headers, "/admin/devices", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
