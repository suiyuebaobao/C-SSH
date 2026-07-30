use std::collections::HashMap;

use cloud_domain::{AppError, AppResult};
use cloud_site_content::SiteContentPayload;
use serde::Deserialize;
use serde_json::Value;

const FIELD_PREFIX: &str = "field:";

#[derive(Debug, Deserialize)]
pub(crate) struct CreateContentForm {
    pub(crate) document_key: cloud_site_content::SiteContentDocumentKey,
    pub(crate) content_locale: cloud_site::Locale,
    pub(crate) lang: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TransitionForm {
    pub(crate) expected_revision: i64,
    pub(crate) lang: Option<String>,
}

pub(crate) fn locale(fields: &HashMap<String, String>) -> cloud_site::Locale {
    super::super::super::shared::locale(fields.get("lang").map(String::as_str))
}

pub(crate) fn expected_revision(fields: &HashMap<String, String>) -> AppResult<i64> {
    fields
        .get("expected_revision")
        .ok_or_else(|| AppError::Validation("缺少 expected_revision".into()))?
        .parse()
        .map_err(|_| AppError::Validation("expected_revision 格式无效".into()))
}

pub(crate) fn apply(
    current: &SiteContentPayload,
    fields: &HashMap<String, String>,
) -> AppResult<SiteContentPayload> {
    for key in fields.keys() {
        if key != "expected_revision" && key != "lang" && !key.starts_with(FIELD_PREFIX) {
            return Err(AppError::Validation("站点内容表单包含未知字段".into()));
        }
    }
    let mut value = serde_json::to_value(current)
        .map_err(|_| AppError::Internal("站点内容无法进入编辑流程".into()))?;
    let mut changed = 0_usize;
    for (key, submitted) in fields {
        let Some(path) = key.strip_prefix(FIELD_PREFIX) else {
            continue;
        };
        if is_structural(path) {
            return Err(AppError::Validation("站点结构字段不可编辑".into()));
        }
        let target = value
            .pointer_mut(path)
            .ok_or_else(|| AppError::Validation("站点内容字段路径无效".into()))?;
        apply_value(target, path, submitted)?;
        changed += 1;
    }
    if changed == 0 {
        return Err(AppError::Validation("没有提交可编辑站点内容".into()));
    }
    serde_json::from_value(value)
        .map_err(|_| AppError::Validation("编辑后的站点内容结构无效".into()))
}

fn apply_value(target: &mut Value, path: &str, submitted: &str) -> AppResult<()> {
    match target {
        Value::String(value) if path.ends_with("/media_slot") => {
            if submitted.is_empty() {
                *target = Value::Null;
            } else if submitted == "home_qr" {
                *value = submitted.to_owned();
            } else {
                return Err(AppError::Validation("受控媒体槽位无效".into()));
            }
        }
        Value::String(value) => *value = submitted.to_owned(),
        Value::Bool(value) => {
            *value = match submitted {
                "true" => true,
                "false" => false,
                _ => return Err(AppError::Validation("布尔字段格式无效".into())),
            };
        }
        Value::Null if path.ends_with("/media_slot") => {
            if submitted.is_empty() {
                *target = Value::Null;
            } else if submitted == "home_qr" {
                *target = Value::String("home_qr".into());
            } else {
                return Err(AppError::Validation("受控媒体槽位无效".into()));
            }
        }
        Value::Null | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            return Err(AppError::Validation("该站点内容字段不可直接编辑".into()));
        }
    }
    Ok(())
}

fn is_structural(path: &str) -> bool {
    path.split('/').any(|segment| {
        matches!(
            segment,
            "document_key" | "schema_version" | "anchor" | "layout" | "tone"
        )
    })
}
