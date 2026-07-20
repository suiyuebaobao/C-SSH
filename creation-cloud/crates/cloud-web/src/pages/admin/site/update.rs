//! 处理站点媒体草稿的双语替代文本更新。
//! 文件身份与发布历史不可原地覆盖，领域用例只接收允许字段。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_site_media::UpdateSiteMediaInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateSiteForm {
    #[serde(default)]
    alt_zh: String,
    #[serde(default)]
    alt_en: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(media_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateSiteForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = UpdateSiteMediaInput {
        alt_zh: shared::optional_text(form.alt_zh),
        alt_en: shared::optional_text(form.alt_en),
    };
    match state.site_media().update(&actor, media_id, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
