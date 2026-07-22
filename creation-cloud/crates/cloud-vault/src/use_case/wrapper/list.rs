//! 从认证会话取得账号身份并分页读取其包装密钥。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{Service, VaultKeyWrapper, repository, wrapper_validation};

impl Service {
    pub async fn list_wrappers(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<VaultKeyWrapper>> {
        let account_id = wrapper_validation::account(session)?;
        repository::wrapper::list::list(&self.pool, account_id, page.normalized()).await
    }
}
