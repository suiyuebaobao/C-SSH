//! 校验全局模型元数据。

use std::collections::HashSet;

use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::types::{
    CreateGlobalModelInput, ModelInterface, ReplaceGlobalModelInput, ValidatedInterfaces,
    ValidatedModel,
};

const MAX_PARAMETERS_BYTES: usize = 16 * 1024;

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
        input.context_length,
        input.interfaces,
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
        input.context_length,
        input.interfaces,
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
    context_length: i32,
    interfaces: Vec<ModelInterface>,
    capability_tags: Vec<String>,
    default_parameters: Value,
    enabled: bool,
    is_default: bool,
    sort_order: i32,
) -> AppResult<ValidatedModel> {
    let name = bounded_text(name, "name", 100)?;
    let provider = slug(provider, "provider", 64)?;
    let interfaces = validate_interfaces(interfaces)?;
    if !(4_096..=2_000_000).contains(&context_length) {
        return Err(AppError::Validation(
            "context_length 必须在 4096 到 2000000 之间".to_owned(),
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
        context_length,
        interfaces,
        capability_tags,
        default_parameters,
        enabled,
        is_default,
        sort_order,
    })
}

fn validate_interfaces(values: Vec<ModelInterface>) -> AppResult<ValidatedInterfaces> {
    if !(1..=2).contains(&values.len()) {
        return Err(AppError::Validation(
            "interfaces 必须包含 1 到 2 个接口".to_owned(),
        ));
    }
    let mut seen = HashSet::new();
    let mut result = ValidatedInterfaces {
        openai_base_url: None,
        openai_model_name: None,
        anthropic_base_url: None,
        anthropic_model_name: None,
    };
    for value in values {
        let api_format = validate_api_format(value.api_format)?;
        if !seen.insert(api_format.clone()) {
            return Err(AppError::Validation(
                "interfaces 中的 api_format 不得重复".to_owned(),
            ));
        }
        let base_url = strict_https_url(value.base_url)?;
        let model_name = bounded_text(value.model_name, "interfaces.model_name", 128)?;
        match api_format.as_str() {
            "openai_compatible" => {
                result.openai_base_url = Some(base_url);
                result.openai_model_name = Some(model_name);
            }
            "anthropic_compatible" => {
                result.anthropic_base_url = Some(base_url);
                result.anthropic_model_name = Some(model_name);
            }
            _ => unreachable!("api_format 已由白名单验证"),
        }
    }
    Ok(result)
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

fn strict_https_url(value: String) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation("base_url 不能为空".to_owned()));
    }
    if value.len() > 512 {
        return Err(AppError::Validation("base_url 过长".to_owned()));
    }
    let parsed =
        Url::parse(value).map_err(|_| AppError::Validation("base_url 格式无效".to_owned()))?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.host_str().is_none()
    {
        return Err(AppError::Validation(
            "base_url 只能是无凭据、query 和 fragment 的 HTTPS 地址".to_owned(),
        ));
    }
    Ok(parsed.to_string().trim_end_matches('/').to_owned())
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
    use serde_json::json;

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

    fn input(interfaces: Vec<ModelInterface>) -> CreateGlobalModelInput {
        CreateGlobalModelInput {
            name: "Example".to_owned(),
            provider: "Example".to_owned(),
            context_length: 128_000,
            interfaces,
            capability_tags: Vec::new(),
            default_parameters: json!({}),
            enabled: true,
            is_default: false,
            sort_order: 0,
        }
    }

    fn interface(api_format: &str, base_url: &str, model_name: &str) -> ModelInterface {
        ModelInterface {
            api_format: api_format.to_owned(),
            base_url: base_url.to_owned(),
            model_name: model_name.to_owned(),
        }
    }

    #[test]
    fn model_requires_one_or_two_unique_interfaces() {
        assert!(create(input(Vec::new())).is_err());
        let duplicate = interface(
            "openai_compatible",
            "https://second.example.test/v1",
            "example-two",
        );
        assert!(
            create(input(vec![
                interface(
                    "openai_compatible",
                    "https://api.example.test/v1",
                    "example-one",
                ),
                duplicate,
            ]))
            .is_err()
        );
    }

    #[test]
    fn model_accepts_distinct_model_ids_for_the_two_interfaces() {
        let validated = create(input(vec![
            interface(
                "openai_compatible",
                "https://api.example.test/v1/",
                "example-openai",
            ),
            interface(
                "anthropic_compatible",
                "https://api.example.test/anthropic",
                "example-anthropic",
            ),
        ]))
        .expect("两个独立接口应有效");
        assert_eq!(
            validated.interfaces.openai_model_name.as_deref(),
            Some("example-openai")
        );
        assert_eq!(
            validated.interfaces.anthropic_model_name.as_deref(),
            Some("example-anthropic")
        );
        assert_eq!(
            validated.interfaces.openai_base_url.as_deref(),
            Some("https://api.example.test/v1")
        );
    }

    #[test]
    fn interface_url_is_https_and_contains_no_credentials_query_or_fragment() {
        for invalid in [
            "http://api.example.test/v1",
            "https://user@example.test/v1",
            "https://api.example.test/v1?key=value",
            "https://api.example.test/v1#fragment",
        ] {
            assert!(
                create(input(vec![interface(
                    "openai_compatible",
                    invalid,
                    "example"
                )]))
                .is_err(),
                "{invalid} 应被拒绝"
            );
        }
    }
}
