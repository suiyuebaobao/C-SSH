//! 校验并规范化用户资料的显示名称与语言字段。

use cloud_domain::{AppError, AppResult};

pub(crate) fn display_name(value: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 80 {
        return Err(AppError::Validation(
            "显示名称长度必须为 1 至 80 个字符".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub(crate) fn locale(value: &str) -> AppResult<String> {
    match value {
        "zh-CN" | "en" => Ok(value.to_owned()),
        _ => Err(AppError::Validation("语言设置无效".to_owned())),
    }
}
