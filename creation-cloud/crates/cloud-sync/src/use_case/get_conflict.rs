//! 按账号所有权读取单个同步冲突。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, SyncConflict, repository, validation};

impl Service {
    pub async fn get_conflict(
        &self,
        account_id: Uuid,
        conflict_id: Uuid,
    ) -> AppResult<SyncConflict> {
        validation::account(account_id)?;
        validation::conflict_id(conflict_id)?;
        repository::get_conflict(&self.pool, account_id, conflict_id).await
    }
}
