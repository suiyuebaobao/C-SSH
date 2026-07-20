//! 处理管理员对单个用户角色或状态的变更。
//! 当前管理员与最后有效管理员保护由管理领域事务再次强制执行。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_admin::{AdminUpdateUserInput, AdminUserRole, AdminUserStatus};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateUserForm {
    role: Option<AdminUserRole>,
    status: Option<AdminUserStatus>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateUserForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .admin()
        .update_user(
            &actor,
            account_id,
            AdminUpdateUserInput {
                role: form.role,
                status: form.status,
            },
        )
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/users", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
