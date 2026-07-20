//! 处理撤销当前首页二维码发布版本的动作。
//! 撤销保留历史记录，不删除同源媒体文件身份。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;
use uuid::Uuid;

use super::super::shared;
use crate::AdminPageState;

#[derive(Debug, Deserialize)]
pub(crate) struct RevokeForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<RevokeForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.site_media().revoke(&actor, media_id).await {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
