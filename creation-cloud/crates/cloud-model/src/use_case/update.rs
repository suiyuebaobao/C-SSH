//! 校验并规范化模型补丁，再执行 update 仓储动作。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{ModelProfile, Service, UpdateModelInput, repository, validation};

impl Service {
    pub async fn update(
        &self,
        account_id: Uuid,
        model_id: Uuid,
        input: UpdateModelInput,
    ) -> AppResult<ModelProfile> {
        validation::account(account_id)?;
        validation::model_id(model_id)?;
        let model = validation::update(input)?;
        repository::update(&self.pool, account_id, model_id, model).await
    }
}
