//! 按账号所有权执行密文信封单查。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, VaultEnvelope, repository, validation};

impl Service {
    pub async fn get(&self, account_id: Uuid, envelope_id: Uuid) -> AppResult<VaultEnvelope> {
        validation::account(account_id)?;
        validation::envelope_id(envelope_id)?;
        repository::get(&self.pool, account_id, envelope_id).await
    }
}
