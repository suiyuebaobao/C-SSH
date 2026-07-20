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
