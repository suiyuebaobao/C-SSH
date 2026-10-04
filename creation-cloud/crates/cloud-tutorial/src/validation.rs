//! 校验教学视频的语言、文本、顺序和CAS输入；正文始终按纯文本处理。

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::model::Metadata;

pub(crate) const MAX_VIDEO_BYTES: u64 = 200 * 1024 * 1024;
pub(crate) const MAX_POSTER_BYTES: usize = 3 * 1024 * 1024;

pub(crate) fn locale(value: &str) -> AppResult<&str> {
    match value {
        "zh-CN" | "en" => Ok(value),
        _ => Err(AppError::Validation("教学视频语言无效".into())),
    }
}

pub(crate) fn id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation("教学视频标识无效".into()));
    }
    Ok(value)
}

pub(crate) fn revision(value: i64) -> AppResult<i64> {
    if value < 1 {
        return Err(AppError::Validation("教学视频修订号无效".into()));
    }
    Ok(value)
}

pub(crate) fn metadata(mut input: Metadata) -> AppResult<Metadata> {
    locale(&input.locale)?;
    input.title = text(&input.title, 160, false, true)?;
    input.description = text(&input.description, 4000, true, false)?;
    input.version_note = text(&input.version_note, 160, false, false)?;
    if !(0..=9999).contains(&input.sort_order) {
        return Err(AppError::Validation("教学视频顺序必须在0至9999之间".into()));
    }
    Ok(input)
}

fn text(value: &str, max: usize, multiline: bool, required: bool) -> AppResult<String> {
    let value = value.trim();
    if (required && value.is_empty())
        || value.chars().count() > max
        || value
            .chars()
            .any(|c| c.is_control() && !(multiline && matches!(c, '\n' | '\t')))
    {
        return Err(AppError::Validation(
            "教学视频文字为空、过长或含控制字符".into(),
        ));
    }
    Ok(value.to_owned())
}
