//! 在管理员二次授权后返回反馈域状态计数，供 API 与 SSR 总览组合使用。

use cloud_domain::{AdminActor, AppResult};

use crate::{FeedbackOverview, Service, authorization, repository};

impl Service {
    pub async fn overview(&self, actor: &AdminActor) -> AppResult<FeedbackOverview> {
        authorization::admin(actor)?;
        repository::overview::execute(&self.pool).await
    }
}
