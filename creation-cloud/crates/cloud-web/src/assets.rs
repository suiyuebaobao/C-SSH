//! 从 crate 内静态文件提供带内容类型与缓存策略的只读资源响应。

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, header},
    response::Response,
    routing::get,
};

const TOKENS_CSS: &str = include_str!("../static/css/tokens.css");
const LAYOUT_CSS: &str = include_str!("../static/css/layout.css");
const COMPONENTS_CSS: &str = include_str!("../static/css/components.css");
const GITHUB_LINK_CSS: &str = include_str!("../static/css/github-link.css");
const PAGES_CSS: &str = include_str!("../static/css/pages.css");
const FEEDBACK_CSS: &str = include_str!("../static/css/feedback.css");
const PUBLIC_DETAIL_CSS: &str = include_str!("../static/css/public-detail.css");
const HOME_FOUNDATION_CSS: &str = include_str!("../static/css/home-foundation.css");
const HOME_HERO_CSS: &str = include_str!("../static/css/home-hero.css");
const HOME_SECTIONS_CSS: &str = include_str!("../static/css/home-sections.css");
const HOME_CLOSING_CSS: &str = include_str!("../static/css/home-closing.css");
const HOME_QR_CSS: &str = include_str!("../static/css/home-qr.css");
const DOCUMENTATION_CSS: &str = include_str!("../static/css/documentation.css");
const ADMIN_FOUNDATION_CSS: &str = include_str!("../static/css/admin-foundation.css");
const ADMIN_COMPONENTS_CSS: &str = include_str!("../static/css/admin-components.css");
const ADMIN_PAGES_CSS: &str = include_str!("../static/css/admin-pages.css");
const SITE_JS: &str = include_str!("../static/js/site.js");
const HOME_QR_JS: &str = include_str!("../static/js/home-qr.js");
const ADMIN_JS: &str = include_str!("../static/js/admin.js");
const FEEDBACK_JS: &str = include_str!("../static/js/feedback.js");
const DOCUMENTATION_SEARCH_JS: &str = include_str!("../static/js/documentation-search.js");
// htmx 2.0.10 按官方 Zero-Clause BSD 许可随源码分发，来源固定到对应版本标签。
// https://github.com/bigskysoftware/htmx/blob/v2.0.10/LICENSE
const HTMX_JS: &str = include_str!("../static/js/htmx.min.js");
const BRAND_IMAGE: &[u8] = include_bytes!("../static/img/brand-c.png");
const PRODUCT_TERMINAL_IMAGE: &[u8] = include_bytes!("../static/img/product-terminal.png");

pub(crate) fn router() -> Router {
    Router::new()
        .route("/static/css/tokens.css", get(tokens_css))
        .route("/static/css/layout.css", get(layout_css))
        .route("/static/css/components.css", get(components_css))
        .route("/static/css/github-link.css", get(github_link_css))
        .route("/static/css/pages.css", get(pages_css))
        .route("/static/css/feedback.css", get(feedback_css))
        .route("/static/css/public-detail.css", get(public_detail_css))
        .route("/static/css/home-foundation.css", get(home_foundation_css))
        .route("/static/css/home-hero.css", get(home_hero_css))
        .route("/static/css/home-sections.css", get(home_sections_css))
        .route("/static/css/home-closing.css", get(home_closing_css))
        .route("/static/css/home-qr.css", get(home_qr_css))
        .route("/static/css/documentation.css", get(documentation_css))
        .route(
            "/static/css/admin-foundation.css",
            get(admin_foundation_css),
        )
        .route(
            "/static/css/admin-components.css",
            get(admin_components_css),
        )
        .route("/static/css/admin-pages.css", get(admin_pages_css))
        .route("/static/js/site.js", get(site_js))
        .route("/static/js/home-qr.js", get(home_qr_js))
        .route("/static/js/admin.js", get(admin_js))
        .route("/static/js/feedback.js", get(feedback_js))
        .route(
            "/static/js/documentation-search.js",
            get(documentation_search_js),
        )
        .route("/static/js/htmx.min.js", get(htmx_js))
        .route("/static/img/brand-c.png", get(brand_image))
        .route(
            "/static/img/product-terminal.png",
            get(product_terminal_image),
        )
}

async fn tokens_css() -> Response {
    text_response(TOKENS_CSS, "text/css; charset=utf-8")
}

async fn layout_css() -> Response {
    text_response(LAYOUT_CSS, "text/css; charset=utf-8")
}

async fn components_css() -> Response {
    text_response(COMPONENTS_CSS, "text/css; charset=utf-8")
}

async fn github_link_css() -> Response {
    text_response(GITHUB_LINK_CSS, "text/css; charset=utf-8")
}

async fn pages_css() -> Response {
    text_response(PAGES_CSS, "text/css; charset=utf-8")
}

async fn feedback_css() -> Response {
    text_response(FEEDBACK_CSS, "text/css; charset=utf-8")
}

async fn public_detail_css() -> Response {
    text_response(PUBLIC_DETAIL_CSS, "text/css; charset=utf-8")
}

async fn home_foundation_css() -> Response {
    text_response(HOME_FOUNDATION_CSS, "text/css; charset=utf-8")
}

async fn home_hero_css() -> Response {
    text_response(HOME_HERO_CSS, "text/css; charset=utf-8")
}

async fn home_sections_css() -> Response {
    text_response(HOME_SECTIONS_CSS, "text/css; charset=utf-8")
}

async fn home_closing_css() -> Response {
    text_response(HOME_CLOSING_CSS, "text/css; charset=utf-8")
}

async fn home_qr_css() -> Response {
    text_response(HOME_QR_CSS, "text/css; charset=utf-8")
}

async fn documentation_css() -> Response {
    text_response(DOCUMENTATION_CSS, "text/css; charset=utf-8")
}

async fn admin_foundation_css() -> Response {
    text_response(ADMIN_FOUNDATION_CSS, "text/css; charset=utf-8")
}

async fn admin_components_css() -> Response {
    text_response(ADMIN_COMPONENTS_CSS, "text/css; charset=utf-8")
}

async fn admin_pages_css() -> Response {
    text_response(ADMIN_PAGES_CSS, "text/css; charset=utf-8")
}

async fn site_js() -> Response {
    text_response(SITE_JS, "text/javascript; charset=utf-8")
}

async fn home_qr_js() -> Response {
    text_response(HOME_QR_JS, "text/javascript; charset=utf-8")
}

async fn admin_js() -> Response {
    text_response(ADMIN_JS, "text/javascript; charset=utf-8")
}

async fn feedback_js() -> Response {
    text_response(FEEDBACK_JS, "text/javascript; charset=utf-8")
}

async fn documentation_search_js() -> Response {
    text_response(DOCUMENTATION_SEARCH_JS, "text/javascript; charset=utf-8")
}

async fn htmx_js() -> Response {
    text_response(HTMX_JS, "text/javascript; charset=utf-8")
}

async fn brand_image() -> Response {
    binary_response(BRAND_IMAGE, "image/png")
}

async fn product_terminal_image() -> Response {
    binary_response(PRODUCT_TERMINAL_IMAGE, "image/png")
}

fn text_response(content: &'static str, content_type: &'static str) -> Response {
    response(Body::from(content), content_type)
}

fn binary_response(content: &'static [u8], content_type: &'static str) -> Response {
    response(Body::from(content), content_type)
}

fn response(body: Body, content_type: &'static str) -> Response {
    let mut response = Response::new(body);
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    response
}
