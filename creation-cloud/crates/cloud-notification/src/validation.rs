//! 校验通知 code/种类、匿名资源身份、分页限制和稳定游标。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{CreateNotificationInput, NotificationKind, NotificationListQuery};

pub(crate) struct ValidatedListQuery {
    pub cursor: Option<(i16, DateTime<Utc>, Uuid)>,
    pub limit: i64,
    pub locale: Locale,
}

#[derive(Clone, Copy)]
pub(crate) enum Locale {
    ZhCn,
    En,
}

pub(crate) fn list_query(input: NotificationListQuery) -> AppResult<ValidatedListQuery> {
    let limit = input.limit.unwrap_or(20);
    if !(1..=50).contains(&limit) {
        return Err(AppError::Validation(
            "通知分页大小必须在 1 到 50 之间".into(),
        ));
    }
    let locale = match input.locale.as_deref().unwrap_or("zh-CN") {
        "zh-CN" => Locale::ZhCn,
        "en" => Locale::En,
        _ => return Err(AppError::Validation("通知语言只允许 zh-CN 或 en".into())),
    };
    let cursor = input
        .cursor
        .as_deref()
        .filter(|value| !value.is_empty())
        .map(parse_cursor)
        .transpose()?;
    Ok(ValidatedListQuery {
        cursor,
        limit: i64::from(limit),
        locale,
    })
}

pub(crate) fn create(input: CreateNotificationInput) -> AppResult<CreateNotificationInput> {
    if input.account_id.is_nil() || input.resource_id.is_some_and(|id| id.is_nil()) {
        return Err(AppError::Validation("通知账号或匿名资源标识无效".into()));
    }
    let valid_code = match input.kind {
        NotificationKind::AccountSecurity => matches!(
            input.code.as_str(),
            "security_review_required" | "password_changed" | "device_revoked" | "session_revoked"
        ),
        NotificationKind::Sync => matches!(
            input.code.as_str(),
            "sync_review_required"
                | "sync_upload_completed"
                | "sync_download_completed"
                | "sync_reset_completed"
        ),
    };
    if !valid_code {
        return Err(AppError::Validation("通知 code 与类型不匹配".into()));
    }
    if input
        .expires_at
        .is_some_and(|expires| expires <= Utc::now())
    {
        return Err(AppError::Validation("通知到期时间必须晚于当前时间".into()));
    }
    Ok(input)
}

pub(crate) fn receipt_revision(revision: i64) -> AppResult<i64> {
    if revision < 1 {
        Err(AppError::Validation("通知修订无效".into()))
    } else {
        Ok(revision)
    }
}

pub(crate) fn priority_rank(value: &str) -> AppResult<i16> {
    match value {
        "normal" => Ok(1),
        "important" => Ok(2),
        "critical" => Ok(3),
        _ => Err(AppError::Storage("数据库中的通知优先级无效".into())),
    }
}

pub(crate) fn encode_cursor(priority_rank: i16, at: DateTime<Utc>, id: Uuid) -> String {
    format!("{priority_rank}.{}.{}", at.timestamp_micros(), id)
}

fn parse_cursor(value: &str) -> AppResult<(i16, DateTime<Utc>, Uuid)> {
    if value.len() > 96 || !value.is_ascii() {
        return Err(AppError::Validation("通知游标无效".into()));
    }
    let (rank, remainder) = value
        .split_once('.')
        .ok_or_else(|| AppError::Validation("通知游标无效".into()))?;
    let (micros, id) = remainder
        .split_once('.')
        .ok_or_else(|| AppError::Validation("通知游标无效".into()))?;
    let rank = rank
        .parse::<i16>()
        .map_err(|_| AppError::Validation("通知游标无效".into()))?;
    if !(1..=3).contains(&rank) {
        return Err(AppError::Validation("通知游标无效".into()));
    }
    let micros = micros
        .parse::<i64>()
        .map_err(|_| AppError::Validation("通知游标无效".into()))?;
    let at = DateTime::from_timestamp_micros(micros)
        .ok_or_else(|| AppError::Validation("通知游标无效".into()))?;
    let id = Uuid::parse_str(id).map_err(|_| AppError::Validation("通知游标无效".into()))?;
    if id.is_nil() {
        return Err(AppError::Validation("通知游标无效".into()));
    }
    Ok((rank, at, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NotificationPriority;

    #[test]
    fn cursor_round_trip_and_query_bounds_are_strict() {
        let at = Utc::now();
        let id = Uuid::now_v7();
        let cursor = encode_cursor(3, at, id);
        let parsed = parse_cursor(&cursor).expect("合法游标应可解析");
        assert_eq!(parsed.0, 3);
        assert_eq!(parsed.1.timestamp_micros(), at.timestamp_micros());
        assert_eq!(parsed.2, id);
        assert!(parse_cursor("bad").is_err());
        assert!(
            list_query(NotificationListQuery {
                cursor: None,
                limit: Some(51),
                locale: None,
            })
            .is_err()
        );
        assert!(
            list_query(NotificationListQuery {
                cursor: Some(String::new()),
                limit: None,
                locale: None,
            })
            .expect("空游标表示第一页")
            .cursor
            .is_none()
        );
    }

    #[test]
    fn code_must_match_notification_kind() {
        let input = CreateNotificationInput {
            account_id: Uuid::now_v7(),
            kind: NotificationKind::Sync,
            priority: NotificationPriority::Normal,
            code: "password_changed".into(),
            resource_id: None,
            expires_at: None,
        };
        assert!(create(input).is_err());
    }
}
