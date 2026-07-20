//! 校验并规范化非敏感模型元数据，再执行 create 仓储动作。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{CreateModelInput, ModelProfile, Service, repository, validation};

impl Service {
    pub async fn create(
        &self,
        account_id: Uuid,
        input: CreateModelInput,
    ) -> AppResult<ModelProfile> {
        validation::account(account_id)?;
        let model = validation::create(input)?;
        repository::create(&self.pool, account_id, model).await
    }
}
