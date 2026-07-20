//! 验证管理员身份和账号标识后读取单个脱敏账号。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{AdminUser, Service, repository, validation};

impl Service {
    pub async fn get_user(&self, actor: &AdminActor, account_id: Uuid) -> AppResult<AdminUser> {
        validation::admin_actor(actor)?;
        repository::users::get::execute(&self.pool, validation::valid_id(account_id, "账号标识")?)
            .await
    }
}
