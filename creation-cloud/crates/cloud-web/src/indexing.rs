//! 提供搜索引擎抓取入口，且只公布当前允许索引的规范页面。

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
};
use cloud_site::{Locale, PageId};

use crate::seo::SeoConfig;

pub(crate) async fn robots(State(config): State<SeoConfig>) -> Response {
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /api/\nDisallow: /console\nDisallow: /health/\nDisallow: /web/auth/\nSitemap: {}\n",
        config.absolute_url("/sitemap.xml")
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

pub(crate) async fn sitemap(State(config): State<SeoConfig>) -> Response {
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    for page in PageId::INDEXABLE {
        if !config.is_indexable(page) {
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
