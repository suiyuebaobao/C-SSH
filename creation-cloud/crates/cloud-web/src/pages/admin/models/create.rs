//! 创建全局模型。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AuthenticatedSession;

use crate::AdminPageState;

use super::{ModelForm, shared};

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<ModelForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = match form.into_create() {
        Ok(input) => input,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.model().create_admin(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/models", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
