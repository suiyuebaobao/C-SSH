//! 分页读取管理端版本目录。

use cloud_domain::{AdminActor, AppResult, Page, PageQuery};

use crate::{Release, Service, authorization, repository};

impl Service {
    pub async fn list_releases(
        &self,
        actor: &AdminActor,
        query: PageQuery,
    ) -> AppResult<Page<Release>> {
        authorization::require(actor)?;
        repository::release::list::execute(&self.pool, query.normalized()).await
    }
}
