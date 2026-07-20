//! 读取版本状态管理概览。

use cloud_domain::{AdminActor, AppResult};

use crate::{ReleaseOverview, Service, repository};

impl Service {
    pub async fn release_overview(&self, actor: &AdminActor) -> AppResult<ReleaseOverview> {
        crate::validation::admin_actor(actor)?;
        repository::overview::releases::execute(&self.pool).await
    }
}
