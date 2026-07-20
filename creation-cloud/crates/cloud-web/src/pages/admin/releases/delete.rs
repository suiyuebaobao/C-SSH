//! 处理删除尚可删除版本的危险动作。
//! 删除资格与关联完整性全部由发布领域用例判定。

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
pub(crate) struct DeleteReleaseForm {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<DeleteReleaseForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.release().delete_release(&actor, release_id).await {
        Ok(()) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
