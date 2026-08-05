//! 处理管理后台永久删除账号；自删、末位管理员与责任归属由领域事务拒绝。

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
pub(crate) struct DeleteUserForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteUserForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.admin().delete_user(&actor, account_id).await {
        Ok(()) => shared::action_success(&headers, "/admin/users", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
