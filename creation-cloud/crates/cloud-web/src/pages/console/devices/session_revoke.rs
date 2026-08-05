//! 撤销本人单个登录会话；撤销当前会话时同时清除浏览器 Cookie 并回到登录页。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::Response,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct RevokeSessionForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(session_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<RevokeSessionForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state.device().revoke_session(&session, session_id).await?;
    if session_id != session.session_id {
        return Ok(common::action_success(&headers, PageId::Devices, locale));
    }

    let mut response = common::action_success_to(&headers, "/login", locale);
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "creation_session=; Path=/; Max-Age=0; Secure; HttpOnly; SameSite=Strict",
        ),
    );
    Ok(response)
}
