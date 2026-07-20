//! 读取用户管理概览。

use cloud_domain::{AdminActor, AppResult};

use crate::{Service, UserOverview, repository};

impl Service {
    pub async fn user_overview(&self, actor: &AdminActor) -> AppResult<UserOverview> {
        crate::validation::admin_actor(actor)?;
        repository::overview::users::execute(&self.pool).await
    }
}
