//! 校验客户端密文信封后执行 create 仓储动作。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{CreateVaultEnvelopeInput, Service, VaultEnvelope, repository, validation};

impl Service {
    pub async fn create(
        &self,
        account_id: Uuid,
        input: CreateVaultEnvelopeInput,
    ) -> AppResult<VaultEnvelope> {
        validation::account(account_id)?;
        let envelope = validation::create(input)?;
        repository::create(&self.pool, account_id, envelope).await
    }
}
