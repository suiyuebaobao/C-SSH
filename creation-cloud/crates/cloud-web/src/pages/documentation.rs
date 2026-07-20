//! 提供中英文入门文档规范路径的 SSR 处理器。

use axum::{extract::State, response::Html};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};

use crate::{SeoConfig, render};

pub(crate) async fn page(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::documentation(PageId::Documentation, Locale::ZhCn, &seo)
}

pub(crate) async fn page_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::documentation(PageId::Documentation, Locale::En, &seo)
}
