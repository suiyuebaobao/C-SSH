//! 处理草稿或校验中版本的资产身份更新。
//! 已发布版本的不可变约束由发布领域用例统一执行。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use cloud_release::UpdateAssetInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateAssetForm {
    #[serde(default)]
    platform: String,
    #[serde(default)]
    architecture: String,
    #[serde(default)]
    package_kind: String,
    #[serde(default)]
    file_name: String,
    byte_size: Option<i64>,
    #[serde(default)]
    sha256: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateAssetForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = UpdateAssetInput {
        platform: shared::optional_text(form.platform),
        architecture: shared::optional_text(form.architecture),
        package_kind: shared::optional_text(form.package_kind),
        file_name: shared::optional_text(form.file_name),
        byte_size: form.byte_size,
        sha256: shared::optional_text(form.sha256),
    };
    match state.release().update_asset(&actor, asset_id, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
