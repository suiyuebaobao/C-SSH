//! 保存公开 SSR 页面读取 SEO 配置与已发布下载清单所需的只读状态。
//! 页面只能调用下载领域公开用例，不跨域查询版本、资产或来源表。

use axum::extract::FromRef;
use cloud_domain::AppResult;
use cloud_download::PublicRelease;
use cloud_site::Locale;

use crate::seo::SeoConfig;

#[derive(Clone)]
pub struct PublicPageState {
    seo: SeoConfig,
    download: cloud_download::Service,
    topics: cloud_seo::Service,
    content: cloud_site_content::Service,
}

impl PublicPageState {
    #[must_use]
    pub const fn new(
        seo: SeoConfig,
        download: cloud_download::Service,
        topics: cloud_seo::Service,
        content: cloud_site_content::Service,
    ) -> Self {
        Self {
            seo,
            download,
            topics,
            content,
        }
    }

    pub(crate) const fn seo(&self) -> &SeoConfig {
        &self.seo
    }

    pub(crate) async fn public_manifest(&self) -> AppResult<Vec<PublicRelease>> {
        self.download.public_manifest().await
    }

    pub(crate) async fn public_topics(&self, locale: Locale) -> AppResult<Vec<String>> {
        let locale = match locale {
            Locale::ZhCn => cloud_seo::SeoLocale::ZhCn,
            Locale::En => cloud_seo::SeoLocale::En,
        };
        Ok(self
            .topics
            .public_topics(locale)
            .await?
            .into_iter()
            .map(|topic| topic.phrase)
            .collect())
    }

    pub(crate) async fn public_content(
        &self,
        key: cloud_site_content::SiteContentDocumentKey,
        locale: Locale,
    ) -> AppResult<cloud_site_content::PublicSiteContent> {
        self.content.public_content(key, locale).await
    }
}

impl FromRef<PublicPageState> for SeoConfig {
    fn from_ref(state: &PublicPageState) -> Self {
        state.seo.clone()
    }
}
