//! 验证管理员身份和设备标识后读取单个设备元数据。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{AdminDevice, Service, repository, validation};

impl Service {
    pub async fn get_device(&self, actor: &AdminActor, device_id: Uuid) -> AppResult<AdminDevice> {
        validation::admin_actor(actor)?;
        repository::devices::get::execute(&self.pool, validation::valid_id(device_id, "设备标识")?)
            .await
    }
}
