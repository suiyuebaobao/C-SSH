//! 验证管理员身份后软撤销指定设备，不删除用户或同步数据。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{AdminDevice, Service, repository, validation};

impl Service {
    pub async fn revoke_device(
        &self,
        actor: &AdminActor,
        device_id: Uuid,
    ) -> AppResult<AdminDevice> {
        validation::admin_actor(actor)?;
        repository::devices::revoke::execute(
            &self.pool,
            validation::valid_id(device_id, "设备标识")?,
        )
        .await
    }
}
