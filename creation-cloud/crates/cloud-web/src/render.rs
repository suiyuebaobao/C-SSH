//! 将内容服务输出映射到职责单一的 Askama 页面模板。

use askama::Template;
use axum::response::Html;
use cloud_domain::{AppError, AppResult};
use cloud_site::{
    DocumentationContent, HomePageContent, Locale, PageId, SiteView, content_service,
};

use crate::seo::{SeoConfig, SeoHead};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    view: SiteView,
    home: HomePageContent,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "marketing.html")]
struct MarketingTemplate {
    view: SiteView,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "documentation.html")]
struct DocumentationTemplate {
    view: SiteView,
    documentation: DocumentationContent,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "feedback.html")]
struct FeedbackTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
    login_href: &'static str,
    github_issues_href: &'static str,
}

#[derive(Template)]
#[template(path = "tutorials.html")]
struct TutorialsTemplate {
    view: SiteView,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "faq.html")]
struct FaqTemplate {
    view: SiteView,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    view: SiteView,
    seo: SeoHead,
    next_path: Option<String>,
}

#[derive(Template)]
#[template(path = "console.html")]
struct ConsoleTemplate {
    view: SiteView,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate {
    view: SiteView,
    seo: SeoHead,
    session_email: Option<String>,
    csrf_token: String,
    is_en: bool,
}

pub(crate) fn home(page: PageId, locale: Locale, config: &SeoConfig) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    let home = view
        .page
        .home_page
        .clone()
        .ok_or_else(|| AppError::Internal("首页内容暂时无法渲染".to_owned()))?;
    render(&HomeTemplate { view, home, seo })
}

pub(crate) fn marketing(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    render(&MarketingTemplate { view, seo })
}

pub(crate) fn documentation(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    let documentation = view
        .page
        .documentation_page
        .clone()
        .ok_or_else(|| AppError::Internal("文档内容暂时无法渲染".to_owned()))?;
    render(&DocumentationTemplate {
        view,
        documentation,
        seo,
    })
}

pub(crate) fn feedback(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    render(&FeedbackTemplate {
        view,
        seo,
        is_en: locale == Locale::En,
        login_href: match locale {
            Locale::ZhCn => "/login?next=%2Ffeedback",
            Locale::En => "/en/login?next=%2Fen%2Ffeedback",
        },
        github_issues_href: "https://github.com/suiyuebaobao/C-SSH/issues/new",
    })
}

pub(crate) fn tutorials(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    render(&TutorialsTemplate { view, seo })
}

pub(crate) fn faq(page: PageId, locale: Locale, config: &SeoConfig) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    render(&FaqTemplate { view, seo })
}

pub(crate) fn account(
    page: PageId,
    locale: Locale,
    next_path: Option<String>,
) -> AppResult<Html<String>> {
    render(&AccountTemplate {
        view: content_service().view(page, locale),
        seo: SeoHead::private(),
        next_path,
    })
}

pub(crate) fn console(page: PageId, locale: Locale) -> AppResult<Html<String>> {
    render(&ConsoleTemplate {
        view: content_service().view(page, locale),
        seo: SeoHead::private(),
    })
}

pub(crate) fn admin(page: PageId, locale: Locale) -> AppResult<Html<String>> {
    render(&AdminTemplate {
        view: content_service().view(page, locale),
        seo: SeoHead::private(),
        session_email: None,
        csrf_token: String::new(),
        is_en: locale == Locale::En,
    })
}

fn public_head(config: &SeoConfig, page: PageId, locale: Locale, view: &SiteView) -> SeoHead {
    SeoHead::public(
        config,
        page,
        locale,
        view.page.meta_title,
        view.page.meta_description,
    )
}

fn render(template: &impl Template) -> AppResult<Html<String>> {
    template
        .render()
        .map(Html)
        .map_err(|_| AppError::Internal("页面暂时无法渲染".to_owned()))
}
