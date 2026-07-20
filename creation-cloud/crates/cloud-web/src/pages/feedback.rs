//! 渲染官网反馈双渠道页面，真实提交由受会话和 CSRF 保护的反馈 API 承担。

use axum::{extract::State, response::Html};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};

use crate::{render, seo::SeoConfig};

pub(crate) async fn page(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::feedback(PageId::Feedback, Locale::ZhCn, &seo)
}

pub(crate) async fn page_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::feedback(PageId::Feedback, Locale::En, &seo)
}
