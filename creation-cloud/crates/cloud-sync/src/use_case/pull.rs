//! 校验增量游标和分页上限，再读取账号范围内的同步记录。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{PullRequest, PullResponse, Service, repository, validation};

impl Service {
    pub async fn pull(&self, account_id: Uuid, request: PullRequest) -> AppResult<PullResponse> {
        validation::account(account_id)?;
        let request = validation::pull(request)?;
        repository::pull(&self.pool, account_id, request).await
    }
}
