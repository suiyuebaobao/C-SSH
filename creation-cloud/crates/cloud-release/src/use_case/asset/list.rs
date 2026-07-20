//! 读取指定版本的全部资产并先确认版本存在。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{ReleaseAsset, Service, authorization, repository, validation};

impl Service {
    pub async fn list_assets(
        &self,
        actor: &AdminActor,
        release_id: Uuid,
    ) -> AppResult<Vec<ReleaseAsset>> {
        authorization::require(actor)?;
        let release_id = validation::valid_id(release_id, "版本标识")?;
        repository::release::get::execute(&self.pool, release_id).await?;
        repository::asset::list::execute(&self.pool, release_id).await
    }
}
