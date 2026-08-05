//! 处理管理员物理删除单个登录会话；删除当前会话时同时清除浏览器 Cookie。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DeleteSessionForm {
    lang: Option<String>,
    account_id: Option<Uuid>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(session_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteSessionForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    match state
        .device()
        .admin_delete_session(&session, session_id)
        .await
    {
        Ok(_) if session_id == session.session_id => {
            let mut response = shared::action_success(&headers, "/login", locale);
            response.headers_mut().append(
                header::SET_COOKIE,
                HeaderValue::from_static(
                    "creation_session=; Path=/; Max-Age=0; Secure; HttpOnly; SameSite=Strict",
                ),
            );
            response
        }
        Ok(_) => {
            let target = form.account_id.map_or_else(
                || "/admin/devices".to_owned(),
                |account_id| format!("/admin/users/{account_id}?tab=devices"),
            );
            shared::action_success(&headers, &target, locale)
        }
        Err(error) => shared::action_error(locale, error),
    }
}
