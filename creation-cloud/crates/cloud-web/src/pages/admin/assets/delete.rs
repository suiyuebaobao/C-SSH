//! 处理删除仍可变安装资产的危险动作。
//! 发布状态与关联约束由发布领域决定，页面层不做旁路清理。

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
pub(crate) struct DeleteAssetForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteAssetForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.release().delete_asset(&actor, asset_id).await {
        Ok(()) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
