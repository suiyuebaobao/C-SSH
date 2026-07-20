//! 把反馈领域对象收敛为管理页面最小摘要与显式详情视图。
//! 列表视图在类型层不持有标题、正文、账号标识或完整请求标识。

use cloud_feedback::{AdminFeedbackDetail, AdminFeedbackSummary, FeedbackStatus};
use cloud_site::Locale;

use super::query;

pub(super) struct FeedbackRow {
    pub(super) id: String,
    pub(super) category: &'static str,
    pub(super) platform: &'static str,
    pub(super) status: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) redacted: bool,
    pub(super) created_at: String,
    pub(super) detail_href: String,
}

pub(super) struct FeedbackDetailView {
    pub(super) id: String,
    pub(super) masked_account_id: String,
    pub(super) request_id: String,
    pub(super) category: &'static str,
    pub(super) platform: &'static str,
    pub(super) app_version: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) status: &'static str,
    pub(super) version: i64,
    pub(super) redacted: bool,
    pub(super) redaction_reason: String,
    pub(super) created_at: String,
    pub(super) updated_at: String,
    pub(super) next_statuses: Vec<StatusChoice>,
    pub(super) page: u32,
    pub(super) size: u32,
    pub(super) status_filter: String,
}

pub(super) struct StatusChoice {
    pub(super) value: &'static str,
    pub(super) label_zh: &'static str,
    pub(super) label_en: &'static str,
}

impl FeedbackRow {
    pub(super) fn new(
        value: AdminFeedbackSummary,
        page: u32,
        size: u32,
        status_filter: Option<FeedbackStatus>,
        locale: Locale,
    ) -> Self {
        Self {
            id: value.id.to_string(),
            category: value.category.as_str(),
            platform: value.platform.as_str(),
            status: value.status.as_str(),
            status_tone: status_tone(value.status),
            redacted: value.redacted,
            created_at: value.created_at.to_rfc3339(),
            detail_href: query::page_href(page, size, status_filter, Some(value.id), locale),
        }
    }
}

impl FeedbackDetailView {
    pub(super) fn new(
        value: AdminFeedbackDetail,
        page: u32,
        size: u32,
        status_filter: Option<FeedbackStatus>,
    ) -> Self {
        Self {
            id: value.id.to_string(),
            masked_account_id: value.masked_account_id,
            request_id: value.request_id,
            category: value.category.as_str(),
            platform: value.platform.as_str(),
            app_version: value.app_version.unwrap_or_else(|| "—".to_owned()),
            title: value.title,
            description: value.description,
            status: value.status.as_str(),
            version: value.version,
            redacted: value.redacted,
            redaction_reason: value.redaction_reason.unwrap_or_default(),
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
            next_statuses: status_choices(value.status),
            page,
            size,
            status_filter: status_filter.map_or("", FeedbackStatus::as_str).to_owned(),
        }
    }
}

const fn status_tone(status: FeedbackStatus) -> &'static str {
    match status {
        FeedbackStatus::New => "warning",
        FeedbackStatus::Triaged | FeedbackStatus::InProgress => "",
        FeedbackStatus::Resolved => "success",
        FeedbackStatus::Closed => "danger",
    }
}

fn status_choices(status: FeedbackStatus) -> Vec<StatusChoice> {
    let values = match status {
        FeedbackStatus::New => &[FeedbackStatus::Triaged, FeedbackStatus::Closed][..],
        FeedbackStatus::Triaged => &[FeedbackStatus::InProgress, FeedbackStatus::Closed][..],
        FeedbackStatus::InProgress => &[FeedbackStatus::Resolved, FeedbackStatus::Closed][..],
        FeedbackStatus::Resolved => &[FeedbackStatus::InProgress, FeedbackStatus::Closed][..],
        FeedbackStatus::Closed => &[],
    };
    values.iter().copied().map(StatusChoice::from).collect()
}

impl From<FeedbackStatus> for StatusChoice {
    fn from(value: FeedbackStatus) -> Self {
        let (label_zh, label_en) = match value {
            FeedbackStatus::New => ("新反馈", "New"),
            FeedbackStatus::Triaged => ("已分诊", "Triaged"),
            FeedbackStatus::InProgress => ("处理中", "In progress"),
            FeedbackStatus::Resolved => ("已解决", "Resolved"),
            FeedbackStatus::Closed => ("已关闭", "Closed"),
        };
        Self {
            value: value.as_str(),
            label_zh,
            label_en,
        }
    }
}
