//! 处理管理后台显式创建用户；密码只在本次表单调用链中传给认证哈希边界。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_admin::{AdminCreateUserInput, AdminUserRole, AdminUserStatus};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateUserForm {
    email: String,
    password: String,
    display_name: String,
    role: Option<AdminUserRole>,
    status: Option<AdminUserStatus>,
    #[serde(default, deserialize_with = "shared::empty_string_as_none")]
    admin_login_name: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateUserForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = AdminCreateUserInput {
        email: form.email,
        password: form.password,
        display_name: form.display_name,
        role: form.role,
        status: form.status,
        admin_login_name: form.admin_login_name,
    };
    match state.admin().create_user(&actor, input).await {
        Ok(user) => shared::action_success(
            &headers,
            &format!("/admin/users/{}?tab=basic", user.id),
            locale,
        ),
        Err(error) => shared::action_error(locale, error),
    }
}
