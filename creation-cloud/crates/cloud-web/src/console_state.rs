//! 集中持有用户控制台读取与变更所需的业务域服务。

#[derive(Clone)]
pub struct ConsolePageState {
    auth: cloud_auth::Service,
    user: cloud_user::Service,
    device: cloud_device::Service,
    host: cloud_host::Service,
    model: cloud_model::Service,
    download: cloud_download::Service,
}

impl ConsolePageState {
    #[must_use]
    pub const fn new(
        auth: cloud_auth::Service,
        user: cloud_user::Service,
        device: cloud_device::Service,
        host: cloud_host::Service,
        model: cloud_model::Service,
        download: cloud_download::Service,
    ) -> Self {
        Self {
            auth,
            user,
            device,
            host,
            model,
            download,
        }
    }

    pub(crate) const fn auth(&self) -> &cloud_auth::Service {
        &self.auth
    }

    pub(crate) const fn user(&self) -> &cloud_user::Service {
        &self.user
    }

    pub(crate) const fn device(&self) -> &cloud_device::Service {
        &self.device
    }

    pub(crate) const fn host(&self) -> &cloud_host::Service {
        &self.host
    }

    pub(crate) const fn model(&self) -> &cloud_model::Service {
        &self.model
    }

    pub(crate) const fn download(&self) -> &cloud_download::Service {
        &self.download
    }
}
