//! 规范化模型元数据，并递归拒绝凭据字段和带凭据的 URL。

use std::collections::{BTreeMap, HashSet};

use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::types::{CreateModel, CreateModelInput, UpdateModel, UpdateModelInput};

const MAX_PARAMETERS_BYTES: usize = 16 * 1024;

pub(crate) fn account(account_id: Uuid) -> AppResult<()> {
    if account_id.is_nil() {
        return Err(AppError::Validation("账号标识不能为空".to_owned()));
    }
    Ok(())
}

pub(crate) fn model_id(model_id: Uuid) -> AppResult<()> {
    if model_id.is_nil() {
        return Err(AppError::Validation("模型标识不能为空".to_owned()));
    }
    Ok(())
}

pub(crate) fn create(input: CreateModelInput) -> AppResult<CreateModel> {
    reject_extra_fields(&input.extra_fields)?;
    let name = bounded_text(input.name, "name", 100)?;
    let provider = slug(input.provider, "provider", 64)?;
    let model_name = bounded_text(input.model_name, "model_name", 128)?;
    let base_url = optional_url(input.base_url)?;
    context_length(input.context_length)?;
    let capability_tags = tags(input.capability_tags)?;
    parameters(&input.default_parameters)?;
    vault_reference(input.vault_envelope_id)?;
    if input.is_default && !input.enabled {
        return Err(AppError::Validation("禁用模型不能设为默认模型".to_owned()));
    }
    Ok(CreateModel {
        id: Uuid::now_v7(),
        name,
        provider,
        base_url,
        model_name,
        context_length: input.context_length,
        capability_tags,
        default_parameters: input.default_parameters,
        enabled: input.enabled,
        is_default: input.is_default,
        sort_order: input.sort_order,
        vault_envelope_id: input.vault_envelope_id,
    })
}

pub(crate) fn update(input: UpdateModelInput) -> AppResult<UpdateModel> {
    reject_extra_fields(&input.extra_fields)?;
    if input.clear_vault_envelope && input.vault_envelope_id.is_some() {
        return Err(AppError::Validation(
            "不能同时设置并清除 vault_envelope_id".to_owned(),
        ));
    }
    let has_change = input.name.is_some()
        || input.provider.is_some()
        || input.base_url.is_some()
        || input.model_name.is_some()
        || input.context_length.is_some()
        || input.capability_tags.is_some()
        || input.default_parameters.is_some()
        || input.enabled.is_some()
        || input.is_default.is_some()
        || input.sort_order.is_some()
        || input.vault_envelope_id.is_some()
        || input.clear_vault_envelope;
    if !has_change {
        return Err(AppError::Validation("模型更新内容不能为空".to_owned()));
    }
    if input.enabled == Some(false) && input.is_default == Some(true) {
        return Err(AppError::Validation("禁用模型不能设为默认模型".to_owned()));
    }

    let name = input
        .name
        .map(|value| bounded_text(value, "name", 100))
        .transpose()?;
    let provider = input
        .provider
        .map(|value| slug(value, "provider", 64))
        .transpose()?;
    let model_name = input
        .model_name
        .map(|value| bounded_text(value, "model_name", 128))
        .transpose()?;
    let base_url = input
        .base_url
        .map(|value| optional_url(Some(value)))
        .transpose()?;
    if let Some(value) = input.context_length {
        context_length(value)?;
    }
    let capability_tags = input.capability_tags.map(tags).transpose()?;
    if let Some(value) = &input.default_parameters {
        parameters(value)?;
    }
    vault_reference(input.vault_envelope_id)?;
    let (enabled, is_default) = match (input.enabled, input.is_default) {
        (Some(false), _) => (Some(false), Some(false)),
        (_, Some(true)) => (Some(true), Some(true)),
        (enabled, is_default) => (enabled, is_default),
    };
    let vault_envelope_id = if input.clear_vault_envelope {
        Some(None)
    } else {
        input.vault_envelope_id.map(Some)
    };
    Ok(UpdateModel {
        name,
        provider,
        base_url,
        model_name,
        context_length: input.context_length,
        capability_tags,
        default_parameters: input.default_parameters,
        enabled,
        is_default,
        sort_order: input.sort_order,
        vault_envelope_id,
    })
}

fn bounded_text(value: String, field: &str, max: usize) -> AppResult<String> {
    let value = value.trim().to_owned();
    if value.is_empty() || value.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{field} 长度必须在 1 到 {max} 个字符之间"
        )));
    }
    Ok(value)
}

fn slug(value: String, field: &str, max: usize) -> AppResult<String> {
    let value = bounded_text(value, field, max)?;
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character))
    {
        return Err(AppError::Validation(format!(
            "{field} 只能包含 ASCII 字母、数字、点、横线和下划线"
        )));
    }
    Ok(value)
}

fn optional_url(value: Option<String>) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() > 512 {
        return Err(AppError::Validation("base_url 过长".to_owned()));
    }
    let parsed =
        Url::parse(value).map_err(|_| AppError::Validation("base_url 格式无效".to_owned()))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.host_str().is_none()
    {
        return Err(AppError::Validation(
            "base_url 只能是无凭据、query 和 fragment 的 HTTP(S) 地址".to_owned(),
        ));
    }
    Ok(Some(parsed.to_string().trim_end_matches('/').to_owned()))
}

fn context_length(value: i32) -> AppResult<()> {
    if !(256..=2_000_000).contains(&value) {
        return Err(AppError::Validation(
            "context_length 必须在 256 到 2000000 之间".to_owned(),
        ));
    }
    Ok(())
}

fn tags(values: Vec<String>) -> AppResult<Vec<String>> {
    if values.len() > 16 {
        return Err(AppError::Validation(
            "capability_tags 最多包含 16 项".to_owned(),
        ));
    }
    let mut unique = HashSet::with_capacity(values.len());
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let value = slug(value, "capability_tags", 32)?;
        if !unique.insert(value.clone()) {
            return Err(AppError::Validation("capability_tags 不得重复".to_owned()));
        }
        normalized.push(value);
    }
    Ok(normalized)
}

fn parameters(value: &Value) -> AppResult<()> {
    reject_sensitive_value(value)?;
    let object = value
        .as_object()
        .ok_or_else(|| AppError::Validation("default_parameters 必须是 JSON object".to_owned()))?;
    let encoded = serde_json::to_vec(value)
        .map_err(|_| AppError::Validation("default_parameters 无法编码".to_owned()))?;
    if encoded.len() > MAX_PARAMETERS_BYTES {
        return Err(AppError::Validation(
            "default_parameters 超过 16 KiB".to_owned(),
        ));
    }
    for (key, value) in object {
        let valid = match key.as_str() {
            "temperature" => number_in(value, 0.0, 2.0),
            "top_p" => number_in(value, 0.0, 1.0),
            "max_tokens" => integer_in(value, 1, 2_000_000),
            "frequency_penalty" | "presence_penalty" => number_in(value, -2.0, 2.0),
            "seed" => value.as_i64().is_some(),
            "parallel_tool_calls" => value.is_boolean(),
            "reasoning_effort" => matches!(value.as_str(), Some("low" | "medium" | "high")),
            _ => false,
        };
        if !valid {
            return Err(AppError::Validation(format!(
                "default_parameters.{key} 不在白名单或值无效"
            )));
        }
    }
    Ok(())
}

fn number_in(value: &Value, minimum: f64, maximum: f64) -> bool {
    value
        .as_f64()
        .is_some_and(|number| number.is_finite() && (minimum..=maximum).contains(&number))
}

fn integer_in(value: &Value, minimum: i64, maximum: i64) -> bool {
    value
        .as_i64()
        .is_some_and(|number| (minimum..=maximum).contains(&number))
}

fn vault_reference(value: Option<Uuid>) -> AppResult<()> {
    if value.is_some_and(|id| id.is_nil()) {
        return Err(AppError::Validation(
            "vault_envelope_id 不能为空 UUID".to_owned(),
        ));
    }
    Ok(())
}

fn reject_extra_fields(fields: &BTreeMap<String, Value>) -> AppResult<()> {
    if let Some(field) = fields.keys().find(|field| is_sensitive_key(field)) {
        return Err(AppError::Validation(format!(
            "模型元数据禁止包含敏感字段 {field}"
        )));
    }
    if let Some(field) = fields.keys().next() {
        return Err(AppError::Validation(format!("未知模型字段 {field}")));
    }
    Ok(())
}

fn reject_sensitive_value(value: &Value) -> AppResult<()> {
    match value {
        Value::Object(entries) => {
            for (key, nested) in entries {
                if is_sensitive_key(key) {
                    return Err(AppError::Validation(format!(
                        "模型元数据禁止包含敏感字段 {key}"
                    )));
                }
                reject_sensitive_value(nested)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_sensitive_value(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "apikey",
        "token",
        "accesstoken",
        "refreshtoken",
        "authorization",
        "password",
        "secret",
        "credential",
        "privatekey",
    ]
    .iter()
    .any(|blocked| normalized.contains(blocked))
}
