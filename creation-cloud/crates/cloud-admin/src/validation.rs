//! 校验审计字段并拒绝可能承载凭据的详情键。

use cloud_domain::{AdminActor, AppError, AppResult, PageQuery};
use serde_json::Value;
use uuid::Uuid;

pub(crate) fn valid_id(value: Uuid, field: &str) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation(format!("{field}不能为空标识")));
    }
    Ok(value)
}

pub(crate) fn admin_actor(actor: &AdminActor) -> AppResult<Uuid> {
    valid_id(actor.account_id(), "管理员身份")
}

pub(crate) fn email_filter(value: &str) -> AppResult<String> {
    if value.len() > 254 {
        return Err(AppError::Validation("邮箱筛选格式无效".to_owned()));
    }
    let value = value.trim().to_lowercase();
    let valid = !value.is_empty()
        && value.len() <= 254
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
    if !valid {
        return Err(AppError::Validation("邮箱筛选格式无效".to_owned()));
    }
    Ok(value)
}

pub(crate) fn page(value: PageQuery) -> PageQuery {
    let value = value.normalized();
    PageQuery {
        page: value.page.min(10_000),
        size: value.size,
    }
}

pub(crate) fn optional_id(value: Option<Uuid>, field: &str) -> AppResult<Option<Uuid>> {
    value.map(|id| valid_id(id, field)).transpose()
}

pub(crate) fn required_code(value: &str, field: &str, max_len: usize) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > max_len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(AppError::Validation(format!("{field}格式无效")));
    }
    Ok(value.to_owned())
}

pub(crate) fn optional_text(
    value: Option<&str>,
    field: &str,
    max_len: usize,
) -> AppResult<Option<String>> {
    value
        .map(|current| {
            let current = current.trim();
            if current.is_empty()
                || current.chars().count() > max_len
                || current.chars().any(char::is_control)
            {
                return Err(AppError::Validation(format!("{field}格式无效")));
            }
            Ok(current.to_owned())
        })
        .transpose()
}

pub(crate) fn details(value: Value) -> AppResult<Value> {
    if !value.is_object() {
        return Err(AppError::Validation("审计详情必须是 JSON 对象".into()));
    }
    if contains_sensitive_key(&value) {
        return Err(AppError::Validation(
            "审计详情不得包含凭据或敏感字段".into(),
        ));
    }
    let encoded = serde_json::to_vec(&value)
        .map_err(|_| AppError::Validation("审计详情无法序列化".into()))?;
    if encoded.len() > 8 * 1024 {
        return Err(AppError::Validation("审计详情不能超过 8 KiB".into()));
    }
    Ok(value)
}

fn contains_sensitive_key(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, child)| {
            let normalized = key
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>();
            matches!(
                normalized.as_str(),
                "password"
                    | "token"
                    | "cookie"
                    | "csrf"
                    | "email"
                    | "body"
                    | "query"
                    | "authorization"
                    | "privatekey"
                    | "apikey"
                    | "secret"
                    | "credential"
                    | "vaultciphertext"
            ) || contains_sensitive_key(child)
        }),
        Value::Array(values) => values.iter().any(contains_sensitive_key),
        _ => false,
    }
}
