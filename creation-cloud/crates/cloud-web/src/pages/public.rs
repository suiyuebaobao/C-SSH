//! 处理首页、产品信息、下载、更新记录与常见问题页面。

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};
use cloud_site_content::{PublicSiteContent, SiteContentDocumentKey, SiteContentPayload};

use crate::{PublicPageState, render, seo::SeoConfig};

pub(crate) async fn home(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::home(PageId::Home, Locale::ZhCn, &seo)
}

pub(crate) async fn home_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::home(PageId::Home, Locale::En, &seo)
}

pub(crate) async fn home_live(State(state): State<PublicPageState>) -> AppResult<Response> {
    live_home(state, Locale::ZhCn).await
}

pub(crate) async fn home_en_live(State(state): State<PublicPageState>) -> AppResult<Response> {
    live_home(state, Locale::En).await
}

async fn live_home(state: PublicPageState, locale: Locale) -> AppResult<Response> {
    let (topics, shell, home) = tokio::try_join!(
        state.public_topics(locale),
        state.public_content(SiteContentDocumentKey::SiteShell, locale),
        state.public_content(SiteContentDocumentKey::Home, locale)
    )?;
    let mut documents = Vec::with_capacity(2);
    let unavailable = collect_public_document(shell, &mut documents)
        | collect_public_document(home, &mut documents);
    if unavailable {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            render::home_unavailable(PageId::Home, locale, documents)?,
        )
            .into_response());
    }
    Ok(render::home_live(PageId::Home, locale, state.seo(), topics, documents)?.into_response())
}

fn collect_public_document(
    content: PublicSiteContent,
    documents: &mut Vec<SiteContentPayload>,
) -> bool {
    match content {
        PublicSiteContent::LegacyFallback => false,
        PublicSiteContent::Published(document) => {
            documents.push(document);
            false
        }
        PublicSiteContent::Unavailable => true,
    }
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

pub(crate) async fn downloads_live(
    State(state): State<PublicPageState>,
) -> AppResult<Html<String>> {
    let manifest = state.public_manifest().await?;
    render::published_catalog(PageId::Downloads, Locale::ZhCn, state.seo(), manifest)
}

pub(crate) async fn downloads_en_live(
    State(state): State<PublicPageState>,
) -> AppResult<Html<String>> {
    let manifest = state.public_manifest().await?;
    render::published_catalog(PageId::Downloads, Locale::En, state.seo(), manifest)
}

pub(crate) async fn changelog(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Changelog, Locale::ZhCn, &seo)
}

pub(crate) async fn changelog_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::marketing(PageId::Changelog, Locale::En, &seo)
}

pub(crate) async fn changelog_live(
    State(state): State<PublicPageState>,
) -> AppResult<Html<String>> {
    let manifest = state.public_manifest().await?;
    render::published_catalog(PageId::Changelog, Locale::ZhCn, state.seo(), manifest)
}

pub(crate) async fn changelog_en_live(
    State(state): State<PublicPageState>,
) -> AppResult<Html<String>> {
    let manifest = state.public_manifest().await?;
    render::published_catalog(PageId::Changelog, Locale::En, state.seo(), manifest)
}

pub(crate) async fn faq(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::faq(PageId::Faq, Locale::ZhCn, &seo)
}

pub(crate) async fn faq_en(State(seo): State<SeoConfig>) -> AppResult<Html<String>> {
    render::faq(PageId::Faq, Locale::En, &seo)
}
