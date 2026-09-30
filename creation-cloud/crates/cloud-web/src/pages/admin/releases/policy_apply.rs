//! 将逐版本单选表单提交到原子策略发布用例，技术校验设置沿用已生效值。

use super::super::shared;
use crate::AdminPageState;
use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppError, AppResult, AuthenticatedSession, normalize_semantic_version};
use cloud_download::{ApplyUpdatePolicyInput, SaveUpdatePolicyDraftInput};
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let locale = shared::locale(form.get("lang").map(String::as_str));
    let result = async {
        let actor = shared::actor_from_session(&session)?;
        let current = state.download().admin_update_policy(&actor).await?;
        let sha256_enabled = if current.published.revision > 0 {
            current.published.sha256_enabled
        } else {
            current.draft.sha256_enabled
        };
        let input = parse_form(&form, sha256_enabled)?;
        state.download().apply_update_policy(&actor, input).await
    }
    .await;
    match result {
        Ok(_) => shared::action_success(&headers, "/admin/releases", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

fn parse_form(
    form: &HashMap<String, String>,
    sha256_enabled: bool,
) -> AppResult<ApplyUpdatePolicyInput> {
    let number = |key| {
        form.get(key)
            .and_then(|value| value.parse::<i64>().ok())
            .ok_or_else(|| AppError::Validation("策略修订无效，请刷新页面".into()))
    };
    let target_release_id = form
        .get("target_release_id")
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<Uuid>()
                .map_err(|_| AppError::Validation("目标版本无效".into()))
        })
        .transpose()?;
    let mut forced_versions = Vec::new();
    let mut disabled_versions = Vec::new();
    let mut no_update_versions = Vec::new();
    for (key, value) in form {
        if let Some(version) = key.strip_prefix("version.") {
            let (version, _) = normalize_semantic_version(version)
                .ok_or_else(|| AppError::Validation("版本号无效".into()))?;
            match value.as_str() {
                "forced" => forced_versions.push(version),
                "disabled" => disabled_versions.push(version),
                "no_update" => no_update_versions.push(version),
                "optional" => {}
                _ => return Err(AppError::Validation("更新方式无效".into())),
            }
        } else if !matches!(
            key.as_str(),
            "expected_revision"
                | "expected_published_revision"
                | "target_release_id"
                | "confirmation"
                | "lang"
        ) {
            return Err(AppError::Validation("更新设置包含未知字段".into()));
        }
    }
    if target_release_id.is_none() && !forced_versions.is_empty() {
        return Err(AppError::Validation("强制更新必须选择目标正式版本".into()));
    }
    Ok(ApplyUpdatePolicyInput {
        expected_published_revision: number("expected_published_revision")?,
        confirmation: form.get("confirmation").cloned().unwrap_or_default(),
        draft: SaveUpdatePolicyDraftInput {
            expected_revision: number("expected_revision")?,
            enabled: target_release_id.is_some(),
            forced_versions,
            disabled_versions: Some(disabled_versions),
            no_update_versions: Some(no_update_versions),
            target_release_id,
            sha256_enabled,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn close_only_and_reopen_forms_preserve_verification_and_require_valid_choices() {
        let mut fields: HashMap<String, String> = [
            ("expected_revision", "2"),
            ("expected_published_revision", "3"),
            ("confirmation", "apply_update_policy"),
            ("version.0.8.8", "disabled"),
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect();
        let input = parse_form(&fields, true).unwrap();
        assert!(!input.draft.enabled);
        assert!(input.draft.sha256_enabled);
        assert_eq!(input.draft.disabled_versions.unwrap(), vec!["0.8.8"]);
        fields.insert("version.0.8.8".into(), "no_update".into());
        let no_update = parse_form(&fields, false).unwrap();
        assert_eq!(no_update.draft.no_update_versions.unwrap(), vec!["0.8.8"]);
        assert!(no_update.draft.disabled_versions.unwrap().is_empty());
        assert!(no_update.draft.forced_versions.is_empty());
        assert!(!no_update.draft.enabled);
        fields.insert("version.0.8.8".into(), "optional".into());
        assert!(
            parse_form(&fields, false)
                .unwrap()
                .draft
                .disabled_versions
                .unwrap()
                .is_empty()
        );
        fields.insert("version.0.8.8".into(), "forced".into());
        assert!(parse_form(&fields, false).is_err());
    }
}
