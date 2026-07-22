//! 修改当前账号密码并复用认证域限流、验密与会话撤销逻辑。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_auth::ChangePassword;
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use serde::Deserialize;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct ChangePasswordForm {
    current_password: String,
    new_password: String,
    revoke_other_sessions: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<ChangePasswordForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state
        .auth()
        .change_password(
            &session,
            ChangePassword {
                current_password: form.current_password,
                new_password: form.new_password,
                revoke_other_sessions: form.revoke_other_sessions.is_some(),
            },
        )
        .await?;
    Ok(common::action_success(&headers, PageId::Profile, locale))
}
