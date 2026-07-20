//! 读取当前账号的未解决同步冲突列表。

use cloud_domain::{AppResult, Page, PageQuery};
use uuid::Uuid;

use crate::{Service, SyncConflict, repository, validation};

impl Service {
    pub async fn list_conflicts(
        &self,
        account_id: Uuid,
        page: PageQuery,
    ) -> AppResult<Page<SyncConflict>> {
        validation::account(account_id)?;
        repository::list_conflicts(&self.pool, account_id, page.normalized()).await
    }
}
