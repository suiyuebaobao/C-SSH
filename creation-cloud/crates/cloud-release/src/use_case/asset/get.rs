//! 校验资产标识并读取单个资产。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{ReleaseAsset, Service, authorization, repository, validation};

impl Service {
    pub async fn get_asset(&self, actor: &AdminActor, id: Uuid) -> AppResult<ReleaseAsset> {
        authorization::require(actor)?;
        repository::asset::get::execute(&self.pool, validation::valid_id(id, "资产标识")?).await
    }
}
