//! 校验来源标识并读取单个来源。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{ReleaseSource, Service, authorization, repository, validation};

impl Service {
    pub async fn get_source(
        &self,
        actor: &AdminActor,
        source_id: Uuid,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        repository::source::get::execute(&self.pool, validation::valid_id(source_id, "来源标识")?)
            .await
    }
}
