//! 从认证会话取得账号身份，校验包装密钥后执行创建仓储动作。

use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{CreateVaultKeyWrapperInput, Service, VaultKeyWrapper, repository, wrapper_validation};

impl Service {
    pub async fn create_wrapper(
        &self,
        session: &AuthenticatedSession,
        input: CreateVaultKeyWrapperInput,
    ) -> AppResult<VaultKeyWrapper> {
        let account_id = wrapper_validation::account(session)?;
        let wrapper = wrapper_validation::create(input)?;
        repository::wrapper::create::create(&self.pool, account_id, wrapper).await
    }
}
