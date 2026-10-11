//! 校验显式修订请求；服务端分配身份，不允许通过普通新建隐式覆盖版本。

use crate::{CreateRevisionInput, Release, Service, authorization, repository, validation};
use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

impl Service {
    pub async fn create_revision(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: CreateRevisionInput,
    ) -> AppResult<Release> {
        authorization::require(actor)?;
        let id = validation::valid_id(id, "源发布标识")?;
        if input.expected_latest_revision < 1 || input.expected_latest_revision == i32::MAX {
            return Err(AppError::Validation("当前修订号无效".into()));
        }
        repository::release::revision::execute(&self.pool, actor.account_id(), id, &input).await
    }
}
