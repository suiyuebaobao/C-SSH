//! 提供搜索引擎抓取入口，且只公布当前允许索引的规范页面。

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};

use crate::{PublicPageState, seo::SeoConfig};

pub(crate) async fn robots(State(config): State<SeoConfig>) -> Response {
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /api/\nDisallow: /console\nDisallow: /health/\nDisallow: /web/auth/\nSitemap: {}\n",
        config.absolute_url("/sitemap.xml")
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

pub(crate) async fn sitemap(State(config): State<SeoConfig>) -> Response {
    sitemap_response(&config, false)
}

pub(crate) async fn sitemap_live(State(state): State<PublicPageState>) -> AppResult<Response> {
    let has_published_catalog = !state.public_manifest().await?.is_empty();
    Ok(sitemap_response(state.seo(), has_published_catalog))
}

fn sitemap_response(config: &SeoConfig, has_published_catalog: bool) -> Response {
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    let mut pages = PageId::INDEXABLE.to_vec();
    if has_published_catalog {
        pages.extend([PageId::Downloads, PageId::Changelog]);
    }
    for page in pages {
        if !config.is_indexable_with_catalog(page, has_published_catalog) {
            continue;
        }
        let zh_url = config.absolute_url(&page.localized_path(Locale::ZhCn));
        let en_url = config.absolute_url(&page.localized_path(Locale::En));
        for current_url in [&zh_url, &en_url] {
            body.push_str("  <url>\n    <loc>");
            body.push_str(&escape_xml(current_url));
            body.push_str("</loc>\n");
            push_alternate(&mut body, "zh-CN", &zh_url);
            push_alternate(&mut body, "en", &en_url);
            push_alternate(&mut body, "x-default", &zh_url);
            body.push_str("  </url>\n");
        }
    }
    body.push_str("</urlset>\n");
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
        .into_response()
}

fn push_alternate(body: &mut String, language: &str, url: &str) {
    body.push_str("    <xhtml:link rel=\"alternate\" hreflang=\"");
    body.push_str(language);
    body.push_str("\" href=\"");
    body.push_str(&escape_xml(url));
    body.push_str("\" />\n");
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use url::Url;

    use super::*;

    #[tokio::test]
    async fn published_catalog_controls_download_and_changelog_sitemap_entries() {
        let config = SeoConfig::from_validated_origin(
            Url::parse("https://cloud.example.com/").expect("测试公开地址应有效"),
            None,
            None,
        );
        let empty = response_text(sitemap_response(&config, false)).await;
        assert_eq!(empty.matches("<url>").count(), 10);
        assert!(!empty.contains("https://cloud.example.com/downloads"));
        assert!(!empty.contains("https://cloud.example.com/changelog"));

        let published = response_text(sitemap_response(&config, true)).await;
        assert_eq!(published.matches("<url>").count(), 14);
        for path in ["downloads", "en/downloads", "changelog", "en/changelog"] {
            assert!(
                published.contains(&format!("https://cloud.example.com/{path}")),
                "站点地图缺少 published 页面：{path}"
            );
        }
    }

    async fn response_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("站点地图正文应可读取");
        String::from_utf8(bytes.to_vec()).expect("站点地图应为 UTF-8")
    }
}
