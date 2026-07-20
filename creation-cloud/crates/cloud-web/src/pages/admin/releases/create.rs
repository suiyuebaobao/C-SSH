//! 处理创建双语发布版本的管理表单。
//! 版本字段与渠道约束由发布领域用例规范化并持久化。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AuthenticatedSession;
use cloud_release::{CreateReleaseInput, ReleaseChannel};
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReleaseForm {
    version: String,
    channel: ReleaseChannel,
    title_zh: String,
    title_en: String,
    notes_zh: String,
    notes_en: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateReleaseForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateReleaseInput {
        version: form.version,
        channel: form.channel,
        title_zh: form.title_zh,
        title_en: form.title_en,
        notes_zh: form.notes_zh,
        notes_en: form.notes_en,
    };
    match state.release().create_release(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
