//! 处理登记不可变安装资产身份的表单。
//! 所属版本状态、平台组合、大小与 SHA256 均由发布领域再次校验。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AuthenticatedSession;
use cloud_release::CreateAssetInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateAssetForm {
    release_id: Uuid,
    platform: String,
    architecture: String,
    package_kind: String,
    file_name: String,
    byte_size: i64,
    sha256: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateAssetForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateAssetInput {
        release_id: form.release_id,
        platform: form.platform,
        architecture: form.architecture,
        package_kind: form.package_kind,
        file_name: form.file_name,
        byte_size: form.byte_size,
        sha256: form.sha256,
    };
    match state.release().create_asset(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
