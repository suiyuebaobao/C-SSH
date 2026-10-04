//! 为Nuxt提供闭集公开页JSON，复用原内容、发布目录及SEO投影；不读取认证上下文。

use axum::{
    Json,
    extract::{Path, Query, State},
    http::header::CACHE_CONTROL,
    response::IntoResponse,
};
use cloud_domain::{AppError, AppResult};
use cloud_site::{Locale, PageId, SiteView, content_service};
use cloud_site_content::{PublicSiteContent, SiteContentDocumentKey};
use serde::{Deserialize, Serialize};

use crate::{
    SiteState,
    catalog::PublishedCatalogView,
    seo::{SeoConfig, SeoHead},
};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageQuery {
    locale: Option<Locale>,
}

#[derive(Serialize)]
pub struct PageDocument {
    view: SiteView,
    seo: SeoHead,
    catalog: Option<PublishedCatalogView>,
    topics: Vec<String>,
}

fn page_id(key: &str) -> AppResult<PageId> {
    match key {
        "home" => Ok(PageId::Home),
        "downloads" => Ok(PageId::Downloads),
        "changelog" => Ok(PageId::Changelog),
        "documentation" => Ok(PageId::Documentation),
        "tutorials" => Ok(PageId::Tutorials),
        "security" => Ok(PageId::Security),
        "faq" => Ok(PageId::Faq),
        "feedback" => Ok(PageId::Feedback),
        _ => Err(AppError::NotFound("公开页面不存在".into())),
    }
}

fn apply_content(view: &mut SiteView, content: PublicSiteContent) -> AppResult<()> {
    match content {
        PublicSiteContent::Published(document) => document.apply(view),
        PublicSiteContent::LegacyFallback => {}
        PublicSiteContent::Unavailable => {
            return Err(AppError::Unavailable("页面内容暂不可用".into()));
        }
    }
    Ok(())
}

pub async fn page(
    State(state): State<SiteState>,
    Path(key): Path<String>,
    Query(query): Query<PageQuery>,
) -> AppResult<impl IntoResponse> {
    let id = page_id(&key)?;
    let locale = query.locale.unwrap_or_default();
    let mut view = content_service().view(id, locale);
    let mut topics = Vec::new();
    if id == PageId::Home {
        let (shell, home, published_topics) = tokio::try_join!(
            state.public_content(SiteContentDocumentKey::SiteShell, locale),
            state.public_content(SiteContentDocumentKey::Home, locale),
            state.public_topics(locale),
        )?;
        apply_content(&mut view, shell)?;
        apply_content(&mut view, home)?;
        topics = published_topics;
    }
    let catalog = if matches!(id, PageId::Downloads | PageId::Changelog) {
        Some(PublishedCatalogView::new(
            state.public_manifest().await?,
            locale,
        ))
    } else {
        None
    };
    let document = document(view, state.seo(), catalog, topics);
    Ok((
        [
            (CACHE_CONTROL.as_str(), "no-store"),
            ("x-robots-tag", "noindex, nofollow"),
        ],
        Json(document),
    ))
}

fn document(
    view: SiteView,
    config: &SeoConfig,
    catalog: Option<PublishedCatalogView>,
    topics: Vec<String>,
) -> PageDocument {
    let seo = SeoHead::public_with_catalog(
        config,
        view.page.id,
        view.shell.locale,
        &view.page.meta_title,
        &view.page.meta_description,
        catalog
            .as_ref()
            .is_some_and(PublishedCatalogView::has_published),
    )
    .with_keywords(&topics);
    PageDocument {
        view,
        seo,
        catalog,
        topics,
    }
}

// 独立原生预览只消费编译内容，不连接数据库，不挂载到正式有状态router。
pub async fn preview(
    State(config): State<SeoConfig>,
    Path(key): Path<String>,
    Query(query): Query<PageQuery>,
) -> AppResult<impl IntoResponse> {
    let id = page_id(&key)?;
    let view = content_service().view(id, query.locale.unwrap_or_default());
    Ok((
        [
            (CACHE_CONTROL.as_str(), "no-store"),
            ("x-robots-tag", "noindex, nofollow"),
        ],
        Json(document(view, &config, None, Vec::new())),
    ))
}
