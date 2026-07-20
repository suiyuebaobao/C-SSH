//! 校验预期 revision 后为密文信封写入墓碑。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{DeleteVaultOutcome, Service, repository, validation};

impl Service {
    pub async fn delete(
        &self,
        account_id: Uuid,
        envelope_id: Uuid,
        expected_revision: i64,
    ) -> AppResult<DeleteVaultOutcome> {
        validation::account(account_id)?;
        validation::envelope_id(envelope_id)?;
        validation::revision(expected_revision)?;
        repository::delete(&self.pool, account_id, envelope_id, expected_revision).await
    }
}
