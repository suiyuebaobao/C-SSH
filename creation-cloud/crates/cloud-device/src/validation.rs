//! 校验并规范化设备名称与客户端公开标识。

use cloud_domain::{AppError, AppResult};

pub(crate) fn name(value: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 80 {
        return Err(AppError::Validation(
            "设备名称长度必须为 1 至 80 个字符".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub(crate) fn public_id(value: &str) -> AppResult<String> {
    let value = value.trim();
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte));
    if !valid {
        return Err(AppError::Validation("设备公开标识格式无效".to_owned()));
    }
    Ok(value.to_owned())
}

pub(crate) fn client_version(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 64 || value.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "客户端版本长度必须为 1 至 64 个非控制字符".to_owned(),
        ));
    }
    Ok(Some(value.to_owned()))
}

pub(crate) fn device_fingerprint(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(AppError::Validation(
            "设备指纹必须是 64 位小写 SHA-256 十六进制字符串".to_owned(),
        ));
    }
    Ok(Some(value.to_owned()))
}

pub(crate) fn user_agent(value: Option<&str>) -> Option<String> {
    let value = value?;
    let sanitized = value
        .chars()
        .filter(|character| !character.is_control())
        .take(512)
        .collect::<String>();
    let sanitized = sanitized.trim();
    (!sanitized.is_empty()).then(|| sanitized.to_owned())
}

pub(crate) fn trusted_ip(value: Option<&str>) -> Option<String> {
    value?
        .trim()
        .parse::<std::net::IpAddr>()
        .ok()
        .map(|address| address.to_string())
}
