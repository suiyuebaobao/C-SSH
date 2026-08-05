use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{
    Announcement, AnnouncementStatus, CreateAnnouncementInput, ReplaceAnnouncementInput,
    model::ValidatedAnnouncement,
};

const MAX_TITLE_CHARS: usize = 160;
const MAX_BODY_CHARS: usize = 10_000;

pub(crate) fn id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation("公告标识无效".to_owned()));
    }
    Ok(value)
}

pub(crate) fn revision(value: i64) -> AppResult<i64> {
    if value < 1 {
        return Err(AppError::Validation(
            "expected_revision 必须大于零".to_owned(),
        ));
    }
    Ok(value)
}

pub(crate) fn create(input: CreateAnnouncementInput) -> AppResult<ValidatedAnnouncement> {
    content(
        input.title_zh_cn,
        input.body_zh_cn,
        input.title_en,
        input.body_en,
    )
}

pub(crate) fn replace(input: ReplaceAnnouncementInput) -> AppResult<(i64, ValidatedAnnouncement)> {
    let expected = revision(input.expected_revision)?;
    let value = content(
        input.title_zh_cn,
        input.body_zh_cn,
        input.title_en,
        input.body_en,
    )?;
    Ok((expected, value))
}

pub(crate) fn editable_draft(record: &Announcement, expected: i64) -> AppResult<()> {
    compare_revision(record, expected)?;
    if record.status != AnnouncementStatus::Draft {
        return Err(AppError::Conflict("只有草稿公告可以编辑或删除".to_owned()));
    }
    Ok(())
}

pub(crate) fn publishable(record: &Announcement, expected: i64) -> AppResult<()> {
    compare_revision(record, expected)?;
    if record.status != AnnouncementStatus::Draft {
        return Err(AppError::Conflict("只有草稿公告可以发布".to_owned()));
    }
    Ok(())
}

pub(crate) fn hideable(record: &Announcement, expected: i64) -> AppResult<()> {
    compare_revision(record, expected)?;
    if record.status != AnnouncementStatus::Published {
        return Err(AppError::Conflict("只有当前已发布公告可以隐藏".to_owned()));
    }
    Ok(())
}

fn compare_revision(record: &Announcement, expected: i64) -> AppResult<()> {
    let expected = revision(expected)?;
    if record.revision != expected {
        return Err(AppError::Conflict(
            "公告已被其他管理员更新，请刷新后重试".to_owned(),
        ));
    }
    Ok(())
}

fn content(
    title_zh_cn: String,
    body_zh_cn: String,
    title_en: String,
    body_en: String,
) -> AppResult<ValidatedAnnouncement> {
    Ok(ValidatedAnnouncement {
        title_zh_cn: title(title_zh_cn, "中文标题")?,
        body_zh_cn: body(body_zh_cn, "中文正文")?,
        title_en: title(title_en, "英文标题")?,
        body_en: body(body_en, "英文正文")?,
    })
}

fn title(value: String, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();
    if !(1..=MAX_TITLE_CHARS).contains(&length) || trimmed.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{label}必须为 1 到 {MAX_TITLE_CHARS} 个可见字符"
        )));
    }
    Ok(trimmed.to_owned())
}

fn body(value: String, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();
    let has_disallowed_control = trimmed
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'));
    if !(1..=MAX_BODY_CHARS).contains(&length) || has_disallowed_control {
        return Err(AppError::Validation(format!(
            "{label}必须为 1 到 {MAX_BODY_CHARS} 个字符"
        )));
    }
    Ok(trimmed.to_owned())
}
