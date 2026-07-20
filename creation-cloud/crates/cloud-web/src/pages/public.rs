//! 处理首页、产品信息、下载、更新记录与常见问题页面。

use axum::{extract::State, response::Html};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};

use crate::{render, seo::SeoConfig};

pub(crate) async fn home(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::home(PageId::Home, Locale::ZhCn, &seo)
}

pub(crate) async fn home_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::home(PageId::Home, Locale::En, &seo)
}

pub(crate) async fn security(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Security, Locale::ZhCn, &seo)
}

pub(crate) async fn security_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Security, Locale::En, &seo)
}

pub(crate) async fn downloads(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Downloads, Locale::ZhCn, &seo)
}

pub(crate) async fn downloads_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Downloads, Locale::En, &seo)
}

pub(crate) async fn changelog(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Changelog, Locale::ZhCn, &seo)
}

pub(crate) async fn changelog_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Changelog, Locale::En, &seo)
}

pub(crate) async fn faq(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::faq(PageId::Faq, Locale::ZhCn, &seo)
}

pub(crate) async fn faq_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::faq(PageId::Faq, Locale::En, &seo)
}
