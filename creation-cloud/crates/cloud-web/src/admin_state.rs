//! 保存管理后台 SSR 页面调用的领域服务与受控运行状态。
//! 状态只负责装配和健康编排，不绕过领域用例对身份与业务约束的复核。

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct AdminHealth {
    pub live: bool,
    pub ready: bool,
    pub database: bool,
    pub downloads: bool,
    pub site_content: bool,
    pub site_media: bool,
    pub environment: String,
}

#[derive(Clone)]
pub struct AdminPageState {
    admin: cloud_admin::Service,
    auth: cloud_auth::Service,
    announcement: cloud_announcement::Service,
    release: cloud_release::Service,
    download: cloud_download::Service,
    feedback: cloud_feedback::Service,
    host: cloud_host::Service,
    model: cloud_model::Service,
    seo: cloud_seo::Service,
    site_content: cloud_site_content::Service,
    site_media: cloud_site_media::Service,
    pool: cloud_store::PgPool,
    environment: String,
}

impl AdminPageState {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        admin: cloud_admin::Service,
        auth: cloud_auth::Service,
        announcement: cloud_announcement::Service,
        release: cloud_release::Service,
        download: cloud_download::Service,
        host: cloud_host::Service,
        model: cloud_model::Service,
        site_content: cloud_site_content::Service,
        site_media: cloud_site_media::Service,
        pool: cloud_store::PgPool,
        environment: String,
    ) -> Self {
        let feedback = cloud_feedback::Service::new(pool.clone());
        let seo = cloud_seo::Service::new(pool.clone());
        Self {
            admin,
            auth,
            announcement,
            release,
            download,
            feedback,
            host,
            model,
            seo,
            site_content,
            site_media,
            pool,
            environment,
        }
    }

    pub(crate) const fn admin(&self) -> &cloud_admin::Service {
        &self.admin
    }

    pub(crate) const fn auth(&self) -> &cloud_auth::Service {
        &self.auth
    }

    pub(crate) const fn announcement(&self) -> &cloud_announcement::Service {
        &self.announcement
    }

    pub(crate) const fn release(&self) -> &cloud_release::Service {
        &self.release
    }

    pub(crate) const fn download(&self) -> &cloud_download::Service {
        &self.download
    }

    pub(crate) const fn feedback(&self) -> &cloud_feedback::Service {
        &self.feedback
    }

    pub(crate) const fn host(&self) -> &cloud_host::Service {
        &self.host
    }

    pub(crate) const fn model(&self) -> &cloud_model::Service {
        &self.model
    }

    pub(crate) const fn seo(&self) -> &cloud_seo::Service {
        &self.seo
    }

    pub(crate) const fn site_media(&self) -> &cloud_site_media::Service {
        &self.site_media
    }

    pub(crate) const fn site_content(&self) -> &cloud_site_content::Service {
        &self.site_content
    }

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
