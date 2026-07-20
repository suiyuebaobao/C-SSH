//! 校验版本标识并读取单个版本。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{Release, Service, authorization, repository, validation};

impl Service {
    pub async fn get_release(&self, actor: &AdminActor, id: Uuid) -> AppResult<Release> {
        authorization::require(actor)?;
        repository::release::get::execute(&self.pool, validation::valid_id(id, "版本标识")?).await
    }
}
