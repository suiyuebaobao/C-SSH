//! 生成公开页面的规范 URL、索引策略与结构化搜索元数据。

use cloud_site::{Locale, PageId};
use serde_json::json;
use url::Url;

#[derive(Clone, Debug)]
pub struct SeoConfig {
    public_base_url: Url,
    google_site_verification: Option<String>,
    baidu_site_verification: Option<String>,
    feedback_enabled: bool,
}

impl SeoConfig {
    #[must_use]
    pub fn for_standalone_preview(public_base_url: Url) -> Self {
        Self {
            public_base_url,
            google_site_verification: None,
            baidu_site_verification: None,
            feedback_enabled: false,
        }
    }

    #[must_use]
    pub fn from_validated_origin(
        public_base_url: Url,
        google_site_verification: Option<String>,
        baidu_site_verification: Option<String>,
    ) -> Self {
        Self {
            public_base_url,
            google_site_verification,
            baidu_site_verification,
            feedback_enabled: true,
        }
    }

    pub(crate) fn absolute_url(&self, path: &str) -> String {
        self.public_base_url
            .join(path)
            .expect("SEO 页面路径必须是受控站内路径")
            .to_string()
    }

    pub(crate) fn origin_url(&self) -> String {
        self.public_base_url.to_string()
    }

    pub(crate) fn is_indexable(&self, page: PageId) -> bool {
        page.is_indexable() && (page != PageId::Feedback || self.feedback_enabled)
    }

    pub(crate) fn is_indexable_with_catalog(
        &self,
        page: PageId,
        has_published_catalog: bool,
    ) -> bool {
        match page {
            PageId::Downloads | PageId::Changelog => has_published_catalog,
            _ => self.is_indexable(page),
        }
    }
}

impl Default for SeoConfig {
    fn default() -> Self {
        Self {
            public_base_url: Url::parse("http://127.0.0.1:8088/")
                .expect("内置本地预览地址必须合法"),
            google_site_verification: None,
            baidu_site_verification: None,
            feedback_enabled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SeoHead {
    pub(crate) robots: &'static str,
    pub(crate) canonical_url: Option<String>,
    pub(crate) alternate_zh_url: Option<String>,
    pub(crate) alternate_en_url: Option<String>,
    pub(crate) alternate_default_url: Option<String>,
    pub(crate) open_graph_title: Option<&'static str>,
    pub(crate) open_graph_description: Option<&'static str>,
    pub(crate) open_graph_url: Option<String>,
    pub(crate) open_graph_locale: Option<&'static str>,
    pub(crate) open_graph_alternate_locale: Option<&'static str>,
    pub(crate) json_ld: Option<String>,
    pub(crate) google_site_verification: Option<String>,
    pub(crate) baidu_site_verification: Option<String>,
}

impl SeoHead {
    pub(crate) fn public(
        config: &SeoConfig,
        page: PageId,
        locale: Locale,
        title: &'static str,
        description: &'static str,
    ) -> Self {
        Self::public_with_indexing(
            config,
            page,
            locale,
            title,
            description,
            config.is_indexable(page),
        )
    }

    pub(crate) fn public_with_catalog(
        config: &SeoConfig,
        page: PageId,
        locale: Locale,
        title: &'static str,
        description: &'static str,
        has_published_catalog: bool,
    ) -> Self {
        Self::public_with_indexing(
            config,
            page,
            locale,
            title,
            description,
            config.is_indexable_with_catalog(page, has_published_catalog),
        )
    }

    fn public_with_indexing(
        config: &SeoConfig,
        page: PageId,
        locale: Locale,
        title: &'static str,
        description: &'static str,
        is_indexable: bool,
    ) -> Self {
        let canonical_url = config.absolute_url(&page.localized_path(locale));
        let alternate_zh_url = config.absolute_url(&page.localized_path(Locale::ZhCn));
        let alternate_en_url = config.absolute_url(&page.localized_path(Locale::En));
        Self {
            robots: if is_indexable {
                "index, follow, max-image-preview:large"
            } else {
                "noindex, follow"
            },
            canonical_url: Some(canonical_url.clone()),
            alternate_zh_url: Some(alternate_zh_url.clone()),
            alternate_en_url: Some(alternate_en_url),
            alternate_default_url: Some(alternate_zh_url),
            open_graph_title: Some(title),
            open_graph_description: Some(description),
            open_graph_url: Some(canonical_url.clone()),
            open_graph_locale: Some(open_graph_locale(locale)),
            open_graph_alternate_locale: Some(open_graph_locale(locale.alternate())),
            json_ld: is_indexable
                .then(|| structured_data(config, page, locale, title, description, &canonical_url)),
            google_site_verification: (page == PageId::Home)
                .then(|| config.google_site_verification.clone())
                .flatten(),
            baidu_site_verification: (page == PageId::Home)
                .then(|| config.baidu_site_verification.clone())
                .flatten(),
        }
    }

    pub(crate) const fn private() -> Self {
        Self {
            robots: "noindex, nofollow",
            canonical_url: None,
            alternate_zh_url: None,
            alternate_en_url: None,
            alternate_default_url: None,
            open_graph_title: None,
            open_graph_description: None,
            open_graph_url: None,
            open_graph_locale: None,
            open_graph_alternate_locale: None,
            json_ld: None,
            google_site_verification: None,
            baidu_site_verification: None,
        }
    }
}

fn structured_data(
    config: &SeoConfig,
    page: PageId,
    locale: Locale,
    title: &str,
    description: &str,
    canonical_url: &str,
) -> String {
    if page == PageId::Home {
        json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": "Creation-SSH",
            "url": config.origin_url(),
        })
        .to_string()
    } else {
        json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": title,
            "description": description,
            "url": canonical_url,
            "inLanguage": locale.code(),
            "isPartOf": {
                "@type": "WebSite",
                "name": "Creation-SSH",
                "url": config.origin_url(),
            },
        })
        .to_string()
    }
}

const fn open_graph_locale(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "zh_CN",
        Locale::En => "en_US",
    }
}
