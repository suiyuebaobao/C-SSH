//! 将内容服务输出映射到职责单一的 Askama 页面模板。

use askama::Template;
use axum::response::Html;
use cloud_domain::{AppError, AppResult};
use cloud_download::{PublicAsset, PublicRelease};
use cloud_site::{
    DocumentationContent, HomePageContent, Locale, PageId, SiteView, content_service,
};
use cloud_site_content::SiteContentPayload;

use crate::seo::{SeoConfig, SeoHead};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    view: SiteView,
    home: HomePageContent,
    seo: SeoHead,
    topics: Vec<String>,
}

#[derive(Template)]
#[template(path = "home-unavailable.html")]
struct HomeUnavailableTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
}

#[derive(Template)]
#[template(path = "marketing.html")]
struct MarketingTemplate {
    view: SiteView,
    seo: SeoHead,
}

#[derive(Template)]
#[template(path = "published-catalog.html")]
struct PublishedCatalogTemplate {
    view: SiteView,
    seo: SeoHead,
    catalog: PublishedCatalogView,
    is_en: bool,
    is_downloads: bool,
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
    admin_login: bool,
    captcha_required: bool,
    captcha_purpose: &'static str,
    captcha_refresh_href: String,
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
    session_identity: Option<String>,
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
    render(&HomeTemplate {
        view,
        home,
        seo,
        topics: Vec::new(),
    })
}

pub(crate) fn home_live(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
    topics: Vec<String>,
    documents: Vec<SiteContentPayload>,
) -> AppResult<Html<String>> {
    let mut view = content_service().view(page, locale);
    for document in documents {
        document.apply(&mut view);
    }
    let seo = public_head(config, page, locale, &view).with_keywords(&topics);
    let home = view
        .page
        .home_page
        .clone()
        .ok_or_else(|| AppError::Internal("首页内容暂时无法渲染".to_owned()))?;
    render(&HomeTemplate {
        view,
        home,
        seo,
        topics,
    })
}

pub(crate) fn home_preview(
    page: PageId,
    locale: Locale,
    topics: Vec<String>,
    documents: Vec<SiteContentPayload>,
) -> AppResult<Html<String>> {
    let mut view = content_service().view(page, locale);
    for document in documents {
        document.apply(&mut view);
    }
    let home = view
        .page
        .home_page
        .clone()
        .ok_or_else(|| AppError::Internal("首页预览内容暂时无法渲染".to_owned()))?;
    render(&HomeTemplate {
        view,
        home,
        seo: SeoHead::private(),
        topics,
    })
}

pub(crate) fn home_unavailable(
    page: PageId,
    locale: Locale,
    documents: Vec<SiteContentPayload>,
) -> AppResult<Html<String>> {
    let mut view = content_service().view(page, locale);
    for document in documents {
        document.apply(&mut view);
    }
    render(&HomeUnavailableTemplate {
        view,
        seo: SeoHead::private(),
        is_en: locale == Locale::En,
    })
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

pub(crate) fn published_catalog(
    page: PageId,
    locale: Locale,
    config: &SeoConfig,
    manifest: Vec<PublicRelease>,
) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let catalog = PublishedCatalogView::new(manifest, locale);
    let seo = SeoHead::public_with_catalog(
        config,
        page,
        locale,
        &view.page.meta_title,
        &view.page.meta_description,
        catalog.has_published(),
    );
    render(&PublishedCatalogTemplate {
        view,
        seo,
        catalog,
        is_en: locale == Locale::En,
        is_downloads: page == PageId::Downloads,
    })
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

pub(crate) fn faq(page: PageId, locale: Locale, config: &SeoConfig) -> AppResult<Html<String>> {
    let view = content_service().view(page, locale);
    let seo = public_head(config, page, locale, &view);
    render(&FaqTemplate { view, seo })
}

pub(crate) fn account(
    page: PageId,
    locale: Locale,
    next_path: Option<String>,
    user_captcha_enabled: bool,
    admin_captcha_enabled: bool,
) -> AppResult<Html<String>> {
    let admin_login = page == PageId::Login
        && next_path
            .as_deref()
            .is_some_and(|path| path == "/admin" || path.starts_with("/admin/"));
    let captcha_required = if admin_login {
        admin_captcha_enabled
    } else {
        user_captcha_enabled
    };
    let captcha_purpose = if admin_login {
        "admin_login"
    } else if page == PageId::Register {
        "register"
    } else {
        "login"
    };
    let captcha_refresh_href = if captcha_required {
        let path = if locale == Locale::En {
            "/en/login"
        } else {
            "/login"
        };
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        if let Some(next_path) = next_path.as_deref() {
            query.append_pair("next", next_path);
        }
        format!("{path}?{}", query.finish())
    } else {
        String::new()
    };
    render(&AccountTemplate {
        view: content_service().view(page, locale),
        seo: SeoHead::private(),
        next_path,
        admin_login,
        captcha_required,
        captcha_purpose,
        captcha_refresh_href,
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
        session_identity: None,
        csrf_token: String::new(),
        is_en: locale == Locale::En,
    })
}

#[derive(Clone, Debug)]
struct PublishedCatalogView {
    releases: Vec<PublishedReleaseView>,
    latest: Option<PublishedReleaseView>,
}

impl PublishedCatalogView {
    fn new(manifest: Vec<PublicRelease>, locale: Locale) -> Self {
        let releases = manifest
            .into_iter()
            .map(|release| PublishedReleaseView::new(release, locale))
            .collect::<Vec<_>>();
        let latest = releases.first().cloned();
        Self { releases, latest }
    }

    fn has_published(&self) -> bool {
        !self.releases.is_empty()
    }
}

#[derive(Clone, Debug)]
struct PublishedReleaseView {
    version: String,
    channel: String,
    title: String,
    notes: String,
    published_at_iso: String,
    published_at_label: String,
    platforms: Vec<PublishedPlatformView>,
    has_windows: bool,
    has_linux: bool,
    has_android: bool,
    has_macos: bool,
    has_ios: bool,
}

impl PublishedReleaseView {
    fn new(release: PublicRelease, locale: Locale) -> Self {
        let title = match locale {
            Locale::ZhCn => release.title_zh,
            Locale::En => release.title_en,
        };
        let notes = match locale {
            Locale::ZhCn => release.notes_zh,
            Locale::En => release.notes_en,
        };
        let published_at_iso = release.published_at.to_rfc3339();
        let published_at_label = release
            .published_at
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
        let mut platforms = Vec::<PublishedPlatformView>::new();
        for asset in release.assets {
            append_asset(&mut platforms, asset);
        }
        platforms.sort_by_key(|platform| platform_rank(&platform.name));
        let has_windows = has_platform(&platforms, "windows");
        let has_linux = has_platform(&platforms, "linux");
        let has_android = has_platform(&platforms, "android");
        let has_macos = has_platform(&platforms, "macos");
        let has_ios = has_platform(&platforms, "ios");
        Self {
            version: release.version,
            channel: release.channel,
            title,
            notes,
            published_at_iso,
            published_at_label,
            platforms,
            has_windows,
            has_linux,
            has_android,
            has_macos,
            has_ios,
        }
    }
}

#[derive(Clone, Debug)]
struct PublishedPlatformView {
    name: String,
    assets: Vec<PublishedAssetView>,
}

#[derive(Clone, Debug)]
struct PublishedAssetView {
    architecture: String,
    package_kind: String,
    download_href: Option<String>,
}

fn append_asset(platforms: &mut Vec<PublishedPlatformView>, asset: PublicAsset) {
    let platform_name = display_platform(&asset.platform);
    let download_href = asset
        .sources
        .into_iter()
        .min_by_key(|source| source.sort_order)
        .map(|source| format!("/api/v1/downloads/{}", source.download_path));
    let asset_view = PublishedAssetView {
        architecture: asset.architecture,
        package_kind: asset.package_kind,
        download_href,
    };
    if let Some(platform) = platforms
        .iter_mut()
        .find(|platform| platform.name.eq_ignore_ascii_case(&platform_name))
    {
        platform.assets.push(asset_view);
    } else {
        platforms.push(PublishedPlatformView {
            name: platform_name,
            assets: vec![asset_view],
        });
    }
}

fn has_platform(platforms: &[PublishedPlatformView], expected: &str) -> bool {
    platforms
        .iter()
        .any(|platform| platform.name.eq_ignore_ascii_case(expected))
}

fn display_platform(platform: &str) -> String {
    match platform.to_ascii_lowercase().as_str() {
        "windows" => "Windows".to_owned(),
        "linux" => "Linux".to_owned(),
        "android" => "Android".to_owned(),
        "macos" => "macOS".to_owned(),
        "ios" => "iOS".to_owned(),
        _ => platform.to_owned(),
    }
}

fn platform_rank(platform: &str) -> usize {
    match platform.to_ascii_lowercase().as_str() {
        "windows" => 0,
        "linux" => 1,
        "android" => 2,
        "macos" => 3,
        "ios" => 4,
        _ => 5,
    }
}

fn public_head(config: &SeoConfig, page: PageId, locale: Locale, view: &SiteView) -> SeoHead {
    SeoHead::public(
        config,
        page,
        locale,
        &view.page.meta_title,
        &view.page.meta_description,
    )
}

fn render(template: &impl Template) -> AppResult<Html<String>> {
    template
        .render()
        .map(Html)
        .map_err(|_| AppError::Internal("页面暂时无法渲染".to_owned()))
}

#[cfg(test)]
mod tests;
