//! 处理为资产新增 HTTPS 外部下载来源的表单。
//! 本站来源只能由流式上传用例创建，页面层不能登记任意本地路径。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_download::{CreateSourceInput, SourceKind};
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateSourceForm {
    provider_name: String,
    external_url: String,
    #[serde(default)]
    sort_order: i32,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<CreateSourceForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateSourceInput {
        asset_id,
        source_kind: SourceKind::External,
        provider_name: form.provider_name,
        local_path: None,
        external_url: Some(form.external_url),
        sort_order: form.sort_order,
        enabled: true,
    };
    match state.download().create_source(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
