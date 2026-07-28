//! 处理 SEO 主题词删除动作。

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

#[derive(Debug, Deserialize)]
pub(crate) struct DeleteSeoTopicForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(topic_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteSeoTopicForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.seo().delete_topic(&actor, topic_id).await {
        Ok(()) => shared::action_success(&headers, "/admin/seo", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
