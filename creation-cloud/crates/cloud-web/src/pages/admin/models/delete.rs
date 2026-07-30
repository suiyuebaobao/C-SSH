//! 删除全局模型。

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

use super::shared;

#[derive(Deserialize)]
pub(crate) struct DeleteModelForm {
    expected_revision: i64,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(model_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteModelForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .model()
        .delete_admin(&actor, model_id, form.expected_revision)
        .await
    {
        Ok(()) => shared::action_success(&headers, "/admin/models", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
