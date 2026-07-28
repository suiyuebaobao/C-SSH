//! 处理 SEO 主题词、排序与启停状态更新。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_seo::{SeoLocale, UpdateSeoTopicInput};
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateSeoTopicForm {
    locale: SeoLocale,
    phrase: String,
    sort_order: i32,
    enabled: bool,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(topic_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateSeoTopicForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = UpdateSeoTopicInput {
        locale: Some(form.locale),
        phrase: Some(form.phrase),
        sort_order: Some(form.sort_order),
        enabled: Some(form.enabled),
    };
    match state.seo().update_topic(&actor, topic_id, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/seo", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
