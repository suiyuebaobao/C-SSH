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
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    email: Option<String>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    display_name: Option<String>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    admin_login_name: Option<String>,
    clear_admin_login_name: Option<String>,
    role: Option<AdminUserRole>,
    #[serde(default, deserialize_with = "super::empty_status_as_none")]
    status: Option<AdminUserStatus>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    new_password: Option<String>,
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
                email: form.email,
                display_name: form.display_name,
                admin_login_name: form.admin_login_name,
                clear_admin_login_name: form.clear_admin_login_name.is_some(),
                role: form.role,
                status: form.status,
                new_password: form.new_password,
            },
        )
        .await
    {
        Ok(_) => shared::action_success(
            &headers,
            &format!("/admin/users/{account_id}?tab=basic"),
            locale,
        ),
        Err(error) => shared::action_error(locale, error),
    }
}
