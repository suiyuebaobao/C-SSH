//! 处理公开教程总览页并将语言参数映射到 Askama 视图。

use axum::{extract::State, response::Html};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};

use crate::{render, seo::SeoConfig};

pub(crate) async fn page(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::tutorials(PageId::Tutorials, Locale::ZhCn, &seo)
}

pub(crate) async fn page_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::tutorials(PageId::Tutorials, Locale::En, &seo)
}
