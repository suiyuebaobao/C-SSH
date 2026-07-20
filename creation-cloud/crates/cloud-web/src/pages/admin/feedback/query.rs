//! 解析管理反馈页的受限筛选、分页与显式详情参数。
//! URL 只携带枚举、分页和反馈 UUID，不承载标题、正文或账号信息。

use cloud_domain::PageQuery;
use cloud_feedback::FeedbackStatus;
use cloud_site::Locale;
use serde::Deserialize;
use uuid::Uuid;

use super::super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct FeedbackQuery {
    pub(super) lang: Option<String>,
    pub(super) page: Option<u32>,
    pub(super) size: Option<u32>,
    pub(super) status: Option<String>,
    pub(super) id: Option<String>,
}

impl FeedbackQuery {
    pub(super) fn locale(&self) -> Locale {
        shared::locale(self.lang.as_deref())
    }

    pub(super) fn page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page.unwrap_or(1),
            size: self.size.unwrap_or(20),
        }
        .normalized()
    }

    pub(super) fn status(&self) -> Result<Option<FeedbackStatus>, ()> {
        optional_status(self.status.as_deref())
    }

    pub(super) fn detail_id(&self) -> Result<Option<Uuid>, ()> {
        match self.id.as_deref().map(str::trim) {
            None | Some("") => Ok(None),
            Some(value) => Uuid::parse_str(value).map(Some).map_err(|_| ()),
        }
    }
}

pub(super) fn parse_status(value: &str) -> Option<FeedbackStatus> {
    match value {
        "new" => Some(FeedbackStatus::New),
        "triaged" => Some(FeedbackStatus::Triaged),
        "in_progress" => Some(FeedbackStatus::InProgress),
        "resolved" => Some(FeedbackStatus::Resolved),
        "closed" => Some(FeedbackStatus::Closed),
        _ => None,
    }
}

pub(super) fn optional_status(value: Option<&str>) -> Result<Option<FeedbackStatus>, ()> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(value) => parse_status(value).map(Some).ok_or(()),
    }
}

pub(super) fn page_href(
    page: u32,
    size: u32,
    status: Option<FeedbackStatus>,
    detail_id: Option<Uuid>,
    locale: Locale,
) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    params.append_pair("page", &page.to_string());
    params.append_pair("size", &size.to_string());
    if let Some(status) = status {
        params.append_pair("status", status.as_str());
    }
    if let Some(detail_id) = detail_id {
        params.append_pair("id", &detail_id.to_string());
    }
    let path = format!("/admin/feedback?{}", params.finish());
    shared::localized_admin_path(&path, locale)
}

pub(super) fn action_return_path(
    id: Uuid,
    page: u32,
    size: u32,
    status: Option<FeedbackStatus>,
) -> String {
    let normalized = PageQuery { page, size }.normalized();
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    params.append_pair("page", &normalized.page.to_string());
    params.append_pair("size", &normalized.size.to_string());
    if let Some(status) = status {
        params.append_pair("status", status.as_str());
    }
    params.append_pair("id", &id.to_string());
    format!("/admin/feedback?{}", params.finish())
}
