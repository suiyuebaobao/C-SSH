use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

pub(crate) fn phrase(value: &str) -> AppResult<String> {
    let normalized = value.trim();
    let length = normalized.chars().count();
    if !(2..=48).contains(&length) {
        return Err(AppError::Validation(
            "SEO 主题词必须为 2 到 48 个 Unicode 字符".to_owned(),
        ));
    }
    if normalized.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "SEO 主题词不能包含控制字符".to_owned(),
        ));
    }
    Ok(normalized.to_owned())
}

pub(crate) fn id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation("SEO 主题词标识无效".to_owned()));
    }
    Ok(value)
}
