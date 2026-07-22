//! 校验 expected_revision 后为当前账号的包装密钥写入软删除墓碑。

use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{DeleteVaultKeyWrapperOutcome, Service, repository, validation, wrapper_validation};

impl Service {
    pub async fn delete_wrapper(
        &self,
        session: &AuthenticatedSession,
        wrapper_id: Uuid,
        expected_revision: i64,
    ) -> AppResult<DeleteVaultKeyWrapperOutcome> {
        let account_id = wrapper_validation::account(session)?;
        wrapper_validation::wrapper_id(wrapper_id)?;
        validation::revision(expected_revision)?;
        repository::wrapper::delete::delete(&self.pool, account_id, wrapper_id, expected_revision)
            .await
    }
}
