//! 读取独立资产页所需的全局分页清单。

use cloud_domain::{AdminActor, AppResult, Page, PageQuery};

use crate::{ReleaseAsset, Service, authorization, repository};

impl Service {
    pub async fn list_all_assets(
        &self,
        actor: &AdminActor,
        query: PageQuery,
    ) -> AppResult<Page<ReleaseAsset>> {
        authorization::require(actor)?;
        repository::asset::list_all::execute(&self.pool, query.normalized()).await
    }
}
