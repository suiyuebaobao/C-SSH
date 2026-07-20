//! 集中校验站点媒体标识、替代文本与列表输入边界。

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

pub(crate) const DEFAULT_LIST_LIMIT: u32 = 50;
pub(crate) const MAX_LIST_LIMIT: u32 = 100;

pub(crate) fn valid_id(id: Uuid) -> AppResult<Uuid> {
    if id.is_nil() {
        return Err(AppError::Validation("站点媒体标识不能为空".into()));
    }
    Ok(id)
}

pub(crate) fn alt_text(value: &str, label: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 || value.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{label}必须为 1 到 200 个无控制字符的文本"
        )));
    }
    Ok(value.to_owned())
}

pub(crate) const fn list_limit(value: Option<u32>) -> u32 {
    match value {
        Some(0) => 1,
        Some(value) if value > MAX_LIST_LIMIT => MAX_LIST_LIMIT,
        Some(value) => value,
        None => DEFAULT_LIST_LIMIT,
    }
}
