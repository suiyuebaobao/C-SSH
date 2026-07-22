//! 保存公开 SSR 页面读取 SEO 配置与已发布下载清单所需的只读状态。
//! 页面只能调用下载领域公开用例，不跨域查询版本、资产或来源表。

use axum::extract::FromRef;
use cloud_domain::AppResult;
use cloud_download::PublicRelease;

use crate::seo::SeoConfig;

#[derive(Clone)]
pub struct PublicPageState {
    seo: SeoConfig,
    download: cloud_download::Service,
}

impl PublicPageState {
    #[must_use]
    pub const fn new(seo: SeoConfig, download: cloud_download::Service) -> Self {
        Self { seo, download }
    }

    pub(crate) const fn seo(&self) -> &SeoConfig {
        &self.seo
    }

    pub(crate) async fn public_manifest(&self) -> AppResult<Vec<PublicRelease>> {
        self.download.public_manifest().await
    }
}

impl FromRef<PublicPageState> for SeoConfig {
    fn from_ref(state: &PublicPageState) -> Self {
        state.seo.clone()
    }
}
