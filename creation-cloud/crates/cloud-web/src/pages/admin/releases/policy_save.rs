//! 将三步版本策略表单转换为共享下载领域草稿输入，不在页面层判断策略。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppError, AuthenticatedSession};
use cloud_download::SaveUpdatePolicyDraftInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Deserialize)]
pub(crate) struct PolicyDraftForm {
    expected_revision: i64,
    enabled: Option<String>,
    forced_versions: String,
    target_release_id: Option<String>,
    sha256_enabled: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<PolicyDraftForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let enabled = form.enabled.as_deref() == Some("true");
    let target_release_id = match parse_target(enabled, form.target_release_id.as_deref()) {
        Ok(value) => value,
        Err(error) => return shared::action_error(locale, error),
    };
    let forced_versions = form
        .forced_versions
        .split(|character: char| character.is_whitespace() || matches!(character, ',' | ';'))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    let input = SaveUpdatePolicyDraftInput {
        expected_revision: form.expected_revision,
        enabled,
        forced_versions,
        target_release_id,
        sha256_enabled: form.sha256_enabled.as_deref() == Some("true"),
    };
    match state
        .download()
        .save_update_policy_draft(&actor, input)
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

fn parse_target(enabled: bool, value: Option<&str>) -> Result<Option<Uuid>, AppError> {
    if !enabled {
        return Ok(None);
    }
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Validation("启用策略必须选择目标正式版本".into()))?
        .parse::<Uuid>()
        .map(Some)
        .map_err(|_| AppError::Validation("目标正式版本无效".into()))
}
