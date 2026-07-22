//! 从管理领域读取实时汇总，并渲染管理后台总览。
//! 数据读取失败时展示明确错误态，不使用占位零值掩盖服务状态。

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_admin::AdminOverview;
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;

use crate::{AdminHealth, AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct OverviewQuery {
    lang: Option<String>,
}

#[derive(Template)]
#[template(path = "admin-overview.html")]
struct OverviewTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    overview: Option<AdminOverview>,
    feedback: Option<cloud_feedback::FeedbackOverview>,
    health: AdminHealth,
    load_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<OverviewQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let actor = shared::actor_from_session(&session)?;
    let (overview_result, feedback_result, health) = tokio::join!(
        state.admin().overview(&actor),
        state.feedback().overview(&actor),
        state.health()
    );
    let (overview, feedback, load_error) = match (overview_result, feedback_result) {
        (Ok(overview), Ok(feedback)) => (Some(overview), Some(feedback), None),
        _ => (None, None, Some(load_error(locale).to_owned())),
    };
    let parts = shared::page_parts(PageId::Admin, locale, &session);
    shared::render(&OverviewTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        overview,
        feedback,
        health,
        load_error,
    })
}

const fn load_error(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "实时总览暂时无法读取，请稍后刷新。",
        Locale::En => "Live overview data is temporarily unavailable. Refresh shortly.",
    }
}
