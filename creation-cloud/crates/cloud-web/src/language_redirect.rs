//! 把旧查询参数语言入口永久归一到稳定的中英文路径。

use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use cloud_site::{Locale, PageId};
use url::form_urlencoded;

pub(crate) async fn english_root_slash() -> Response {
    (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, "/en")]).into_response()
}

pub(crate) async fn legacy_documentation() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/docs/getting-started")],
    )
        .into_response()
}

pub(crate) async fn legacy_documentation_en() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/en/docs/getting-started")],
    )
        .into_response()
}

pub(crate) async fn canonicalize(request: Request, next: Next) -> Response {
    if request.method() == Method::GET
        && let Some(page) = public_page(request.uri().path())
        && let Some(language) = requested_language(request.uri().query())
    {
        let target = page.localized_path(Locale::from_code(Some(&language)));
        return (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, target)]).into_response();
    }
    next.run(request).await
}

fn requested_language(query: Option<&str>) -> Option<String> {
    form_urlencoded::parse(query?.as_bytes())
        .find(|(name, _)| name.eq_ignore_ascii_case("lang"))
        .map(|(_, value)| value.into_owned())
}

const fn public_page(path: &str) -> Option<PageId> {
    match path.as_bytes() {
        b"/" => Some(PageId::Home),
        b"/docs/getting-started"
        | b"/en/docs/getting-started"
        | b"/platforms"
        | b"/en/platforms"
        | b"/features"
        | b"/en/features" => Some(PageId::Documentation),
        b"/tutorials" | b"/en/tutorials" => Some(PageId::Documentation),
        b"/security" => Some(PageId::Security),
        b"/downloads" => Some(PageId::Downloads),
        b"/changelog" => Some(PageId::Changelog),
        b"/faq" => Some(PageId::Faq),
        b"/feedback" => Some(PageId::Feedback),
        b"/login" => Some(PageId::Login),
        b"/register" => Some(PageId::Register),
        _ => None,
    }
}
