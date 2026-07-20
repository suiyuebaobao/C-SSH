//! 校验完整替换信封与预期 revision 后执行 update 仓储动作。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, UpdateVaultEnvelopeInput, VaultEnvelope, repository, validation};

impl Service {
    pub async fn update(
        &self,
        account_id: Uuid,
        envelope_id: Uuid,
        input: UpdateVaultEnvelopeInput,
    ) -> AppResult<VaultEnvelope> {
        validation::account(account_id)?;
        validation::envelope_id(envelope_id)?;
        let envelope = validation::update(input)?;
        repository::update(&self.pool, account_id, envelope_id, envelope).await
    }
}
