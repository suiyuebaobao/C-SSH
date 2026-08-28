//! 将管理员二次确认提交到版本策略追加式发布用例。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AuthenticatedSession;
use cloud_download::PublishUpdatePolicyInput;
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

#[derive(Deserialize)]
pub(crate) struct PublishPolicyForm {
    expected_draft_revision: i64,
    confirmation: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<PublishPolicyForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .download()
        .publish_update_policy(
            &actor,
            PublishUpdatePolicyInput {
                expected_draft_revision: form.expected_draft_revision,
                confirmation: form.confirmation,
            },
        )
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
