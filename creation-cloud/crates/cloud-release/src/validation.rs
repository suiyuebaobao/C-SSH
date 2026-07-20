//! 集中校验版本和资产输入，避免 handler 与 SQL 重复业务规则。

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

pub(crate) const MAX_ASSET_BYTES: i64 = 4 * 1024 * 1024 * 1024;

pub(crate) fn required_text(value: &str, field: &str, max_len: usize) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Validation(format!("{field}不能为空")));
    }
    if value.chars().count() > max_len {
        return Err(AppError::Validation(format!(
            "{field}长度不能超过{max_len}"
        )));
    }
    Ok(value.to_owned())
}

pub(crate) fn optional_text(
    value: Option<&str>,
    field: &str,
    max_len: usize,
) -> AppResult<Option<String>> {
    value
        .map(|current| required_text(current, field, max_len))
        .transpose()
}

pub(crate) fn valid_id(value: Uuid, field: &str) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation(format!("{field}不能为空标识")));
    }
    Ok(value)
}

pub(crate) fn file_name(value: &str) -> AppResult<String> {
    let value = required_text(value, "文件名", 255)?;
    if value == "."
        || value == ".."
        || value.contains(['/', '\\'])
        || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation("文件名必须是安全的单一文件名".into()));
    }
    Ok(value)
}

pub(crate) fn byte_size(value: i64) -> AppResult<i64> {
    if !(1..=MAX_ASSET_BYTES).contains(&value) {
        return Err(AppError::Validation(format!(
            "资产大小必须大于 0 且不能超过 {MAX_ASSET_BYTES} 字节"
        )));
    }
    Ok(value)
}

pub(crate) fn platform(value: &str) -> AppResult<String> {
    let value = required_text(value, "平台", 32)?.to_ascii_lowercase();
    match value.as_str() {
        "windows" | "linux" | "android" => Ok(value),
        _ => Err(AppError::Validation(
            "发布资产平台只允许 windows、linux 或 android".into(),
        )),
    }
}

pub(crate) fn architecture(value: &str) -> AppResult<String> {
    let value = required_text(value, "架构", 32)?.to_ascii_lowercase();
    match value.as_str() {
        "x86_64" | "aarch64" => Ok(value),
        _ => Err(AppError::Validation(
            "发布资产架构只允许 x86_64 或 aarch64".into(),
        )),
    }
}

pub(crate) fn package_kind(value: &str) -> AppResult<String> {
    let value = required_text(value, "包类型", 32)?.to_ascii_lowercase();
    match value.as_str() {
        "exe" | "msi" | "zip" | "appimage" | "deb" | "apk" | "aab" => Ok(value),
        _ => Err(AppError::Validation("发布资产包类型不在允许列表内".into())),
    }
}

pub(crate) fn asset_identity(platform: &str, package_kind: &str) -> AppResult<()> {
    let valid = matches!(
        (platform, package_kind),
        ("windows", "exe" | "msi" | "zip")
            | ("linux", "appimage" | "deb")
            | ("android", "apk" | "aab")
    );
    if !valid {
        return Err(AppError::Validation("平台与包类型组合无效".into()));
    }
    Ok(())
}

pub(crate) fn sha256(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() != 64 || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::Validation(
            "SHA256 必须是 64 位十六进制字符串".into(),
        ));
    }
    Ok(normalized)
}
