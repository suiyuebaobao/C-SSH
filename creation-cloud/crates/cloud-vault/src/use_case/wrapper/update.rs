//! 校验 revision 与完整包装密钥密文后执行乐观锁更新。

use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, UpdateVaultKeyWrapperInput, VaultKeyWrapper, repository, wrapper_validation};

impl Service {
    pub async fn update_wrapper(
        &self,
        session: &AuthenticatedSession,
        wrapper_id: Uuid,
        input: UpdateVaultKeyWrapperInput,
    ) -> AppResult<VaultKeyWrapper> {
        let account_id = wrapper_validation::account(session)?;
        wrapper_validation::wrapper_id(wrapper_id)?;
        let wrapper = wrapper_validation::update(input)?;
        repository::wrapper::update::update(&self.pool, account_id, wrapper_id, wrapper).await
    }
}
