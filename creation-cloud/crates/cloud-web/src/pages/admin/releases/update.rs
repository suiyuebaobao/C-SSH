//! 处理版本双语元数据更新或单向状态迁移。
//! 页面不自行放宽状态机，发布领域会再次校验每次迁移。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_release::{ReleaseStatus, UpdateReleaseInput};
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateReleaseForm {
    #[serde(default)]
    title_zh: String,
    #[serde(default)]
    title_en: String,
    #[serde(default)]
    notes_zh: String,
    #[serde(default)]
    notes_en: String,
    status: Option<ReleaseStatus>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(release_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateReleaseForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = UpdateReleaseInput {
        title_zh: shared::optional_text(form.title_zh),
        title_en: shared::optional_text(form.title_en),
        notes_zh: shared::optional_text(form.notes_zh),
        notes_en: shared::optional_text(form.notes_en),
        status: form.status,
    };
    match state
        .release()
        .update_release(&actor, release_id, input)
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
