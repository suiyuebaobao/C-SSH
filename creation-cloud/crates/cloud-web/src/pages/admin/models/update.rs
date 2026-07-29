//! 更新全局模型。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use uuid::Uuid;

use crate::AdminPageState;

use super::{ModelForm, shared};

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<ModelForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = match form.into_replace() {
        Ok(input) => input,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.model().replace_admin(&actor, model_id, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/models", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
