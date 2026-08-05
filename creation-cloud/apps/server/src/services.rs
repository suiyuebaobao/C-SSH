//! 一次构造 HTTP、维护 supervisor 与 CLI 共用的全部可克隆服务句柄。

use cloud_config::CloudConfig;
use cloud_store::PgPool;
use std::sync::Arc;

use crate::smtp_mailer::SmtpVerificationMailer;

#[derive(Clone)]
pub struct AppServices {
    pub pool: PgPool,
    pub admin: cloud_admin::Service,
    pub announcement: cloud_announcement::Service,
    pub auth: cloud_auth::Service,
    pub device: cloud_device::Service,
    pub download: cloud_download::Service,
    pub feedback: cloud_feedback::Service,
    pub host: cloud_host::Service,
    pub maintenance: cloud_maintenance::Service,
    pub model: cloud_model::Service,
    pub release: cloud_release::Service,
    pub seo: cloud_seo::Service,
    pub site_content: cloud_site_content::Service,
    pub site_media: cloud_site_media::Service,
    pub sync: cloud_sync::Service,
    pub user: cloud_user::Service,
}

impl AppServices {
    pub fn new(pool: PgPool, config: &CloudConfig) -> anyhow::Result<Self> {
        let site_media =
            cloud_site_media::Service::new(pool.clone(), config.site_media_root.clone());
        let site_content = cloud_site_content::Service::new(pool.clone(), site_media.clone());
        let auth = match config.smtp.as_ref() {
            Some(smtp) => cloud_auth::Service::with_verification(
                pool.clone(),
                config.session_ttl,
                smtp.verification_key().to_vec(),
                Arc::new(SmtpVerificationMailer::new(
                    smtp,
                    config.public_base_url.as_str(),
                )?),
            ),
            None => cloud_auth::Service::new(pool.clone(), config.session_ttl),
        };
        Ok(Self {
            admin: cloud_admin::Service::new(pool.clone()),
            announcement: cloud_announcement::Service::new(pool.clone()),
            auth,
            device: cloud_device::Service::new(pool.clone()),
            download: cloud_download::Service::new(pool.clone(), config.download_root.clone()),
            feedback: cloud_feedback::Service::new(pool.clone()),
            host: cloud_host::Service::new(pool.clone()),
            maintenance: cloud_maintenance::Service::new(pool.clone()),
            model: cloud_model::Service::new(pool.clone()),
            release: cloud_release::Service::new(pool.clone()),
            seo: cloud_seo::Service::new(pool.clone()),
            site_content,
            site_media,
            sync: cloud_sync::Service::new(pool.clone()),
            user: cloud_user::Service::new(pool.clone()),
            pool,
        })
    }
}
