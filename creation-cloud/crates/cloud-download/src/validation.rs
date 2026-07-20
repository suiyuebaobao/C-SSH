//! 校验来源输入、相对路径和不含查询数据的 HTTPS 外链。

use cloud_domain::{AppError, AppResult};
use url::Url;
use uuid::Uuid;

pub(crate) fn valid_id(value: Uuid, field: &str) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation(format!("{field}不能为空标识")));
    }
    Ok(value)
}

pub(crate) fn required_text(value: &str, field: &str, max_len: usize) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field}不能为空")));
    }
    if value.chars().count() > max_len || value.chars().any(char::is_control) {
        return Err(AppError::Validation(format!("{field}格式或长度无效")));
    }
    Ok(value.to_owned())
}

pub(crate) fn sort_order(value: i32) -> AppResult<i32> {
    if !(0..=100_000).contains(&value) {
        return Err(AppError::Validation(
            "来源排序必须在 0 到 100000 之间".into(),
        ));
    }
    Ok(value)
}

pub(crate) fn local_path(value: &str) -> AppResult<String> {
    let normalized = value.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains(':')
        || normalized.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.chars().any(char::is_control)
        })
    {
        return Err(AppError::Validation(
            "本站来源必须是下载根目录内的规范相对路径".into(),
        ));
    }
    Ok(normalized)
}

pub(crate) fn external_url(value: &str) -> AppResult<String> {
    let parsed = Url::parse(value.trim())
        .map_err(|_| AppError::Validation("外部来源 URL 格式无效".into()))?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(AppError::Validation("外部来源只允许 HTTPS URL".into()));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() || parsed.fragment().is_some() {
        return Err(AppError::Validation(
            "外部来源 URL 不得包含凭据或片段".into(),
        ));
    }
    if parsed.query().is_some() {
        return Err(AppError::Validation("外部来源 URL 不得包含查询参数".into()));
    }
    Ok(parsed.to_string())
}
