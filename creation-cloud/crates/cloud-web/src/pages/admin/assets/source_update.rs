//! 处理下载来源排序或启停状态更新。
//! 来源位置不可原地覆盖，下载领域只允许更新受控字段。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_download::UpdateSourceInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateSourceForm {
    sort_order: Option<i32>,
    enabled: Option<bool>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(source_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateSourceForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .download()
        .update_source(
            &actor,
            source_id,
            UpdateSourceInput {
                sort_order: form.sort_order,
                enabled: form.enabled,
            },
        )
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
