//! 分页执行当前账号的密文信封列表查询。

use cloud_domain::{AppResult, Page, PageQuery};
use uuid::Uuid;

use crate::{Service, VaultEnvelope, repository, validation};

impl Service {
    pub async fn list(&self, account_id: Uuid, page: PageQuery) -> AppResult<Page<VaultEnvelope>> {
        validation::account(account_id)?;
        repository::list(&self.pool, account_id, page.normalized()).await
    }
}
