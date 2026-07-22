//! 一次构造 HTTP、维护 supervisor 与 CLI 共用的全部可克隆服务句柄。

use cloud_config::CloudConfig;
use cloud_store::PgPool;

#[derive(Clone)]
pub struct AppServices {
    pub pool: PgPool,
    pub admin: cloud_admin::Service,
    pub auth: cloud_auth::Service,
    pub device: cloud_device::Service,
    pub download: cloud_download::Service,
    pub feedback: cloud_feedback::Service,
    pub maintenance: cloud_maintenance::Service,
    pub model: cloud_model::Service,
    pub release: cloud_release::Service,
    pub site_media: cloud_site_media::Service,
    pub sync: cloud_sync::Service,
    pub user: cloud_user::Service,
    pub vault: cloud_vault::Service,
}

impl AppServices {
    #[must_use]
    pub fn new(pool: PgPool, config: &CloudConfig) -> Self {
        Self {
            admin: cloud_admin::Service::new(pool.clone()),
            auth: cloud_auth::Service::new(pool.clone(), config.session_ttl),
            device: cloud_device::Service::new(pool.clone()),
            download: cloud_download::Service::new(pool.clone(), config.download_root.clone()),
            feedback: cloud_feedback::Service::new(pool.clone()),
            maintenance: cloud_maintenance::Service::new(pool.clone()),
            model: cloud_model::Service::new(pool.clone()),
            release: cloud_release::Service::new(pool.clone()),
            site_media: cloud_site_media::Service::new(
                pool.clone(),
                config.site_media_root.clone(),
            ),
            sync: cloud_sync::Service::new(pool.clone()),
            user: cloud_user::Service::new(pool.clone()),
            vault: cloud_vault::Service::new(pool.clone()),
            pool,
        }
    }
}
