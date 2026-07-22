//! 规范化反馈纯文本，并拒绝控制字符、明显凭据标记与真实 IPv4 字面量。

use std::net::Ipv4Addr;

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{
    AdminFeedbackListQuery, CreateFeedbackInput, RedactFeedbackInput, UpdateFeedbackStatusInput,
};

const PRIVATE_KEY_MARKERS: [&str; 4] = [
    "-----begin private key-----",
    "-----begin rsa private key-----",
    "-----begin ec private key-----",
    "-----begin openssh private key-----",
];
const ASSIGNMENT_KEYS: [&str; 11] = [
    "password",
    "passwd",
    "pwd",
    "token",
    "api_key",
    "apikey",
    "secret",
    "authorization",
    "密码",
    "口令",
    "令牌",
];

pub(crate) fn create(mut input: CreateFeedbackInput) -> AppResult<CreateFeedbackInput> {
    if !input.redaction_confirmed {
        return Err(AppError::Validation(
            "提交前必须确认正文已经脱敏".to_owned(),
        ));
    }
    input.title = text(&input.title, "反馈标题", 5, 120, false)?;
    input.description = text(&input.description, "反馈描述", 20, 4000, true)?;
    input.app_version = app_version(input.app_version.as_deref())?;
    reject_sensitive(&input.title)?;
    reject_sensitive(&input.description)?;
    Ok(input)
}

pub(crate) fn id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation("反馈标识不能为空".to_owned()));
    }
    Ok(value)
}

pub(crate) fn status(mut input: UpdateFeedbackStatusInput) -> AppResult<UpdateFeedbackStatusInput> {
    expected_version(input.expected_version)?;
    input.reason = audit_reason(&input.reason, "状态变更原因")?;
    Ok(input)
}

pub(crate) fn management_query(mut query: AdminFeedbackListQuery) -> AdminFeedbackListQuery {
    query.page = bounded_page(query.page);
    query
}

pub(crate) fn bounded_page(page: cloud_domain::PageQuery) -> cloud_domain::PageQuery {
    let page = page.normalized();
    cloud_domain::PageQuery {
        page: page.page.min(10_000),
        size: page.size,
    }
}

pub(crate) fn redaction(mut input: RedactFeedbackInput) -> AppResult<RedactFeedbackInput> {
    expected_version(input.expected_version)?;
    input.reason = audit_reason(&input.reason, "脱敏原因")?;
    Ok(input)
}

fn audit_reason(value: &str, field: &str) -> AppResult<String> {
    let value = text(value, field, 5, 500, false)?;
    reject_sensitive(&value)?;
    if value.contains('@') {
        return Err(AppError::Validation(format!(
            "{field}不得包含账号邮箱，请使用无身份信息的摘要"
        )));
    }
    Ok(value)
}

fn expected_version(value: i64) -> AppResult<()> {
    if value < 1 {
        return Err(AppError::Validation("反馈期望版本必须大于零".to_owned()));
    }
    Ok(())
}

fn text(
    value: &str,
    field: &str,
    min_len: usize,
    max_len: usize,
    multiline: bool,
) -> AppResult<String> {
    let value = value.trim();
    let length = value.chars().count();
    let has_disallowed_control = value
        .chars()
        .any(|character| character.is_control() && !(multiline && "\n\r\t".contains(character)));
    if length < min_len || length > max_len || has_disallowed_control {
        return Err(AppError::Validation(format!("{field}长度或字符格式无效")));
    }
    Ok(value.to_owned())
}

fn app_version(value: Option<&str>) -> AppResult<Option<String>> {
    value
        .map(|value| {
            let value = value.trim();
            if value.is_empty()
                || value.chars().count() > 32
                || !value.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-')
                })
            {
                return Err(AppError::Validation("应用版本格式无效".to_owned()));
            }
            Ok(value.to_owned())
        })
        .transpose()
}

fn reject_sensitive(value: &str) -> AppResult<()> {
    let lower = value.to_lowercase();
    if (lower.contains("-----begin ") && lower.contains(" private key-----"))
        || lower.contains("-----begin pgp private key block-----")
        || PRIVATE_KEY_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        || ASSIGNMENT_KEYS
            .iter()
            .any(|key| contains_assignment(&lower, key))
        || contains_bearer_value(&lower)
    {
        return Err(AppError::Validation(
            "反馈不得包含私钥、Token 或密码等凭据".to_owned(),
        ));
    }
    if contains_non_example_ipv4(value) {
        return Err(AppError::Validation(
            "反馈不得包含真实 IPv4 地址，请改用 RFC 5737 示例地址".to_owned(),
        ));
    }
    Ok(())
}

fn contains_assignment(value: &str, key: &str) -> bool {
    let mut remaining = value;
    while let Some(position) = remaining.find(key) {
        let after_key = &remaining[position + key.len()..];
        let trimmed = after_key.trim_start();
        if trimmed
            .strip_prefix('=')
            .or_else(|| trimmed.strip_prefix(':'))
            .is_some_and(|assigned| !assigned.trim().is_empty())
        {
            return true;
        }
        remaining = after_key;
    }
    false
}

fn contains_bearer_value(value: &str) -> bool {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|pair| {
            pair.first() == Some(&"bearer") && pair.get(1).is_some_and(|token| !token.is_empty())
        })
}

fn contains_non_example_ipv4(value: &str) -> bool {
    value
        .split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .filter(|candidate| candidate.matches('.').count() == 3)
        .filter_map(|candidate| candidate.parse::<Ipv4Addr>().ok())
        .any(|address| !allowed_ipv4(address))
}

const fn allowed_ipv4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 127
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
}

#[cfg(test)]
mod tests;
