//! 按账号所有权执行模型单查。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{ModelProfile, Service, repository, validation};

impl Service {
    pub async fn get(&self, account_id: Uuid, model_id: Uuid) -> AppResult<ModelProfile> {
        validation::account(account_id)?;
        validation::model_id(model_id)?;
        repository::get(&self.pool, account_id, model_id).await
    }
}
