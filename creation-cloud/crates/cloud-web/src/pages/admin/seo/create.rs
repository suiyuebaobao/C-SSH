//! 处理 SEO 主题词新增表单。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AuthenticatedSession;
use cloud_seo::{CreateSeoTopicInput, SeoLocale};
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateSeoTopicForm {
    locale: SeoLocale,
    phrase: String,
    sort_order: i32,
    enabled: bool,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateSeoTopicForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateSeoTopicInput {
        locale: form.locale,
        phrase: form.phrase,
        sort_order: form.sort_order,
        enabled: form.enabled,
    };
    match state.seo().create_topic(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/seo", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
