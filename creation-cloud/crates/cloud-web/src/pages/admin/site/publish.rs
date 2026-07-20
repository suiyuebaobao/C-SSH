//! 处理将首页二维码草稿发布为当前同源资源的动作。
//! 同槽位切换和状态约束由站点媒体领域事务完成。

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
pub(crate) struct PublishForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<PublishForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.site_media().publish(&actor, media_id).await {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
