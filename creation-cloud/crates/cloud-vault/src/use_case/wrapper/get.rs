//! 从认证会话取得账号身份并读取其名下包装密钥。

use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, VaultKeyWrapper, repository, wrapper_validation};

impl Service {
    pub async fn get_wrapper(
        &self,
        session: &AuthenticatedSession,
        wrapper_id: Uuid,
    ) -> AppResult<VaultKeyWrapper> {
        let account_id = wrapper_validation::account(session)?;
        wrapper_validation::wrapper_id(wrapper_id)?;
        repository::wrapper::get::get(&self.pool, account_id, wrapper_id).await
    }
}
