//! 处理删除仍允许删除的站点媒体草稿。
//! 已发布或已撤销历史能否删除由站点媒体领域用例决定。

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
pub(crate) struct DeleteForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.site_media().delete(&actor, media_id).await {
        Ok(()) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
