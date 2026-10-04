//! 聚合领域就绪探针，供纯API服务的健康检查与管理员总览使用。

use cloud_store::PgPool;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AdminHealth {
    pub live: bool,
    pub ready: bool,
    pub database: bool,
    pub downloads: bool,
    pub site_content: bool,
    pub site_media: bool,
    pub environment: String,
}

#[derive(Clone)]
pub(crate) struct HealthState {
    pub pool: PgPool,
    pub download: cloud_download::Service,
    pub site_content: cloud_site_content::Service,
    pub site_media: cloud_site_media::Service,
    pub environment: String,
}

impl HealthState {
    pub async fn health(&self) -> AdminHealth {
        let (database, downloads, site_content, site_media) = tokio::join!(
            cloud_store::health(&self.pool),
            self.download.ready(),
            self.site_content.ready(),
            self.site_media.ready()
        );
        let database = database.is_ok();
        let downloads = downloads.is_ok();
        let site_content = site_content.is_ok();
        let site_media = site_media.is_ok();
        AdminHealth {
            live: true,
            ready: database && downloads && site_content && site_media,
            database,
            downloads,
            site_content,
            site_media,
            environment: self.environment.clone(),
        }
    }
}
