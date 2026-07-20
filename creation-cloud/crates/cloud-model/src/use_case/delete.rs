//! 按账号所有权执行模型删除。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, repository, validation};

impl Service {
    pub async fn delete(&self, account_id: Uuid, model_id: Uuid) -> AppResult<()> {
        validation::account(account_id)?;
        validation::model_id(model_id)?;
        repository::delete(&self.pool, account_id, model_id).await
    }
}
