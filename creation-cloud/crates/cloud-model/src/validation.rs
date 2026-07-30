//! 校验全局模型元数据与不透明客户端密文。

use std::collections::HashSet;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::types::{CreateGlobalModelInput, ReplaceGlobalModelInput, ValidatedModel};

const MAX_PARAMETERS_BYTES: usize = 16 * 1024;
const MAX_SECRET_BYTES: usize = 1024 * 1024;

pub(crate) fn model_id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        Err(AppError::Validation("模型标识不能为空".to_owned()))
    } else {
        Ok(value)
    }
}

pub(crate) fn revision(value: i64) -> AppResult<i64> {
    if value <= 0 {
        Err(AppError::Validation(
            "expected_revision 必须大于 0".to_owned(),
        ))
    } else {
        Ok(value)
    }
}

pub(crate) fn create(input: CreateGlobalModelInput) -> AppResult<ValidatedModel> {
    validate_model(
        input.name,
        input.provider,
        input.api_format,
        input.base_url,
        input.model_name,
        input.context_length,
        input.capability_tags,
        input.default_parameters,
        input.enabled,
        input.is_default,
        input.sort_order,
    )
}

pub(crate) fn replace(input: ReplaceGlobalModelInput) -> AppResult<(i64, ValidatedModel)> {
    let expected_revision = revision(input.expected_revision)?;
    let model = validate_model(
        input.name,
        input.provider,
        input.api_format,
        input.base_url,
        input.model_name,
        input.context_length,
        input.capability_tags,
        input.default_parameters,
        input.enabled,
        input.is_default,
        input.sort_order,
    )?;
    Ok((expected_revision, model))
}

#[allow(clippy::too_many_arguments)]
fn validate_model(
    name: String,
    provider: String,
    api_format: String,
    base_url: Option<String>,
    model_name: String,
    context_length: i32,
    capability_tags: Vec<String>,
    default_parameters: Value,
    enabled: bool,
    is_default: bool,
    sort_order: i32,
) -> AppResult<ValidatedModel> {
    let name = bounded_text(name, "name", 100)?;
    let provider = slug(provider, "provider", 64)?;
    let api_format = validate_api_format(api_format)?;
    let model_name = bounded_text(model_name, "model_name", 128)?;
    let base_url = optional_url(base_url)?
        .ok_or_else(|| AppError::Validation("base_url 不能为空".to_owned()))?;
    if !(256..=2_000_000).contains(&context_length) {
        return Err(AppError::Validation(
            "context_length 必须在 256 到 2000000 之间".to_owned(),
        ));
    }
    if !enabled && is_default {
        return Err(AppError::Validation("禁用模型不能设为默认模型".to_owned()));
    }
    let capability_tags = tags(capability_tags)?;
    parameters(&default_parameters)?;
    Ok(ValidatedModel {
        name,
        provider,
        api_format,
        base_url: Some(base_url),
        model_name,
        context_length,
        capability_tags,
        default_parameters,
        enabled,
        is_default,
        sort_order,
    })
}

fn validate_api_format(value: String) -> AppResult<String> {
    let value = value.trim().to_ascii_lowercase();
    if matches!(value.as_str(), "openai_compatible" | "anthropic_compatible") {
        Ok(value)
    } else {
        Err(AppError::Validation(
            "api_format 只能是 openai_compatible 或 anthropic_compatible".to_owned(),
        ))
    }
}

pub(crate) fn ciphertext(value: &str) -> AppResult<Vec<u8>> {
    let decoded = STANDARD
        .decode(value)
        .map_err(|_| AppError::Validation("ciphertext 必须是规范 base64".to_owned()))?;
    if !(16..=MAX_SECRET_BYTES).contains(&decoded.len()) {
        return Err(AppError::Validation(
            "ciphertext 解码后必须在 16 字节到 1 MiB 之间".to_owned(),
        ));
    }
    if STANDARD.encode(&decoded) != value {
        return Err(AppError::Validation(
            "ciphertext 必须使用规范 base64 编码".to_owned(),
        ));
    }
    Ok(decoded)
}

fn bounded_text(value: String, field: &str, max: usize) -> AppResult<String> {
    let value = value.trim().to_owned();
    if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{field} 必须是 1 到 {max} 个无控制字符的文本"
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

fn tags(values: Vec<String>) -> AppResult<Vec<String>> {
    if values.len() > 16 {
        return Err(AppError::Validation(
            "capability_tags 最多 16 项".to_owned(),
        ));
    }
    let mut seen = HashSet::new();
    let mut result = Vec::with_capacity(values.len());
    for value in values {
        let value = slug(value, "capability_tags", 32)?;
        if !seen.insert(value.clone()) {
            return Err(AppError::Validation("capability_tags 不得重复".to_owned()));
        }
        result.push(value);
    }
    Ok(result)
}

fn parameters(value: &Value) -> AppResult<()> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::Validation("default_parameters 必须是对象".to_owned()))?;
    if serde_json::to_vec(value)
        .map_err(|_| AppError::Validation("default_parameters 无法编码".to_owned()))?
        .len()
        > MAX_PARAMETERS_BYTES
    {
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
            "reasoning_effort" => {
                matches!(value.as_str(), Some("low" | "medium" | "high" | "xhigh"))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_format_accepts_only_the_two_supported_interfaces() {
        assert_eq!(
            validate_api_format(" OpenAI_Compatible ".to_owned()).expect("OpenAI 兼容格式应有效"),
            "openai_compatible"
        );
        assert_eq!(
            validate_api_format("anthropic_compatible".to_owned())
                .expect("Anthropic 兼容格式应有效"),
            "anthropic_compatible"
        );
        assert!(validate_api_format("claude".to_owned()).is_err());
        assert!(validate_api_format("custom".to_owned()).is_err());
    }

    #[test]
    fn ciphertext_is_canonical_and_bounded() {
        let encoded = STANDARD.encode([7_u8; 32]);
        assert_eq!(ciphertext(&encoded).expect("密文应有效"), vec![7_u8; 32]);
        assert!(ciphertext("not base64").is_err());
        assert!(ciphertext(&STANDARD.encode([0_u8; 4])).is_err());
    }
}
