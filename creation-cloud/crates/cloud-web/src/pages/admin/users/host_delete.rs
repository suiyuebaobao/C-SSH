//! 处理管理员在指定账号范围内永久删除一台云端主机。

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

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DeleteHostForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((account_id, host_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Form(form): Form<DeleteHostForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .host()
        .admin_delete_for_user(&actor, account_id, host_id)
        .await
    {
        Ok(()) => shared::action_success(
            &headers,
            &format!("/admin/users/{account_id}?tab=hosts"),
            locale,
        ),
        Err(error) => shared::action_error(locale, error),
    }
}
