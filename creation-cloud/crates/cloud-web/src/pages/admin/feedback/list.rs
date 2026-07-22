//! 读取带状态筛选的管理反馈摘要，并按显式 UUID 单查加载完整详情。
//! 列表调用不会读取标题正文；详情错误与列表错误分别呈现且不填充假数据。

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_feedback::{AdminFeedbackListQuery, FeedbackStatus};
use cloud_site::{Locale, PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::{
    super::shared,
    query::{self, FeedbackQuery},
    view::{FeedbackDetailView, FeedbackRow},
};

#[derive(Template)]
#[template(path = "admin-feedback-list.html")]
pub(super) struct FeedbackTemplate {
    pub(super) view: SiteView,
    pub(super) seo: SeoHead,
    pub(super) session_identity: Option<String>,
    pub(super) csrf_token: String,
    pub(super) is_en: bool,
    pub(super) rows: Vec<FeedbackRow>,
    pub(super) detail: Option<FeedbackDetailView>,
    pub(super) list_error: Option<String>,
    pub(super) detail_error: Option<String>,
    pub(super) status_filter: String,
    pub(super) page_number: u32,
    pub(super) page_size: u32,
    pub(super) total: i64,
    pub(super) previous_href: Option<String>,
    pub(super) next_href: Option<String>,
    pub(super) close_detail_href: String,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<FeedbackQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.page_query();
    let status_filter = query.status();
    let (rows, total, list_error, selected_status) = match status_filter {
        Ok(status) => match state
            .feedback()
            .list_feedback_for_management(
                &actor,
                AdminFeedbackListQuery {
                    page: page_query,
                    status,
                },
            )
            .await
        {
            Ok(page) => (
                page.items
                    .into_iter()
                    .map(|item| {
                        FeedbackRow::new(item, page_query.page, page_query.size, status, locale)
                    })
                    .collect(),
                page.total,
                None,
                status,
            ),
            Err(_) => (
                Vec::new(),
                0,
                Some(list_load_error(locale).to_owned()),
                status,
            ),
        },
        Err(()) => (
            Vec::new(),
            0,
            Some(invalid_status_error(locale).to_owned()),
            None,
        ),
    };
    let (detail, detail_error) = load_detail(
        &state,
        &actor,
        query.detail_id(),
        page_query.page,
        page_query.size,
        selected_status,
        locale,
    )
    .await;
    let previous_href = (list_error.is_none() && page_query.page > 1).then(|| {
        query::page_href(
            page_query.page - 1,
            page_query.size,
            selected_status,
            None,
            locale,
        )
    });
    let next_href = (list_error.is_none()
        && i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| {
            query::page_href(
                page_query.page + 1,
                page_query.size,
                selected_status,
                None,
                locale,
            )
        });
    let close_detail_href = query::page_href(
        page_query.page,
        page_query.size,
        selected_status,
        None,
        locale,
    );
    let parts = shared::page_parts(PageId::AdminFeedback, locale, &session);
    shared::render(&FeedbackTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        detail,
        list_error,
        detail_error,
        status_filter: selected_status
            .map_or("", FeedbackStatus::as_str)
            .to_owned(),
        page_number: page_query.page,
        page_size: page_query.size,
        total,
        previous_href,
        next_href,
        close_detail_href,
    })
}

async fn load_detail(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
    detail_id: Result<Option<uuid::Uuid>, ()>,
    page: u32,
    size: u32,
    status_filter: Option<FeedbackStatus>,
    locale: Locale,
) -> (Option<FeedbackDetailView>, Option<String>) {
    let id = match detail_id {
        Ok(Some(id)) => id,
        Ok(None) => return (None, None),
        Err(()) => return (None, Some(invalid_detail_error(locale).to_owned())),
    };
    match state
        .feedback()
        .get_feedback_for_management(actor, id)
        .await
    {
        Ok(detail) => (
            Some(FeedbackDetailView::new(detail, page, size, status_filter)),
            None,
        ),
        Err(error) => (None, Some(detail_load_error(locale, &error).to_owned())),
    }
}

const fn list_load_error(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "反馈列表暂时无法读取，请稍后刷新。",
        Locale::En => "Feedback is temporarily unavailable. Refresh shortly.",
    }
}

const fn invalid_status_error(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "反馈状态筛选值无效，请重新选择。",
        Locale::En => "The feedback status filter is invalid. Choose a listed status.",
    }
}

const fn invalid_detail_error(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "详情链接中的反馈 UUID 无效。",
        Locale::En => "The feedback UUID in the detail link is invalid.",
    }
}

const fn detail_load_error(locale: Locale, error: &AppError) -> &'static str {
    match (locale, error) {
        (Locale::ZhCn, AppError::NotFound(_)) => "该反馈已不存在。",
        (Locale::En, AppError::NotFound(_)) => "This feedback no longer exists.",
        (Locale::ZhCn, _) => "反馈详情暂时无法读取，请稍后重试。",
        (Locale::En, _) => "Feedback details are temporarily unavailable. Try again shortly.",
    }
}
