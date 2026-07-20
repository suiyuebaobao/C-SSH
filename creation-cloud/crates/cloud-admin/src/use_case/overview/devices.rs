//! 读取设备管理概览。

use cloud_domain::{AdminActor, AppResult};

use crate::{DeviceOverview, Service, repository};

impl Service {
    pub async fn device_overview(&self, actor: &AdminActor) -> AppResult<DeviceOverview> {
        crate::validation::admin_actor(actor)?;
        repository::overview::devices::execute(&self.pool).await
    }
}
