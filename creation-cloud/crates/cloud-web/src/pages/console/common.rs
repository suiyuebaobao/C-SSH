//! 提供控制台页面共用的私有 SEO、分页与安全重定向辅助函数。

use askama::Template;
use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use cloud_domain::{AppError, AppResult, PageQuery};
use cloud_site::{Locale, PageId, SiteView, content_service};

use crate::seo::SeoHead;

const HX_REQUEST: &str = "hx-request";
const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");

pub(super) const fn first_page() -> PageQuery {
    PageQuery { page: 1, size: 100 }
}

pub(super) fn view(page: PageId, locale: Locale) -> SiteView {
    content_service().view(page, locale)
}

pub(super) const fn seo() -> SeoHead {
    SeoHead::private()
}

pub(super) const fn is_en(locale: Locale) -> bool {
    matches!(locale, Locale::En)
}

pub(super) fn locale(value: Option<&str>) -> Locale {
    Locale::from_code(value)
}

pub(super) fn action_success(headers: &HeaderMap, page: PageId, locale: Locale) -> Response {
    let target = page.localized_path(locale);
    if headers
        .get(HX_REQUEST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        let mut response = StatusCode::OK.into_response();
        if let Ok(value) = HeaderValue::from_str(&target) {
            response.headers_mut().insert(HX_REDIRECT, value);
        }
        response
    } else {
        Redirect::to(&target).into_response()
    }
}

pub(super) fn render(template: &impl Template) -> AppResult<Html<String>> {
    template
        .render()
        .map(Html)
        .map_err(|_| AppError::Internal("控制台页面暂时无法渲染".to_owned()))
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderValue, header::LOCATION};

    use super::*;

    #[test]
    fn write_success_uses_htmx_redirect_and_preserves_locale() {
        let mut headers = HeaderMap::new();
        headers.insert(HX_REQUEST, HeaderValue::from_static("true"));

        let response = action_success(&headers, PageId::Profile, Locale::En);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[HX_REDIRECT], "/console/profile?lang=en");
        assert!(!response.headers().contains_key(LOCATION));
    }

    #[test]
    fn write_success_keeps_standard_redirect_for_non_htmx_requests() {
        let response = action_success(&HeaderMap::new(), PageId::Devices, Locale::ZhCn);

        assert!(response.status().is_redirection());
        assert_eq!(response.headers()[LOCATION], "/console/devices");
    }
}
