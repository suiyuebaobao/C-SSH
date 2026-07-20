//! 读取指定资产的全部来源并先确认资产存在。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{ReleaseSource, Service, authorization, repository, validation};

impl Service {
    pub async fn list_sources(
        &self,
        actor: &AdminActor,
        asset_id: Uuid,
    ) -> AppResult<Vec<ReleaseSource>> {
        authorization::require(actor)?;
        let asset_id = validation::valid_id(asset_id, "资产标识")?;
        repository::asset::get(&self.pool, asset_id).await?;
        repository::source::list::execute(&self.pool, asset_id).await
    }
}
