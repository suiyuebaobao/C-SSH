//! 校验增量游标和分页上限，再读取账号范围内的同步记录。

use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{PullRequest, PullResponse, Service, limiter::AccessKind, repository, validation};

impl Service {
    pub async fn pull(
        &self,
        session: &AuthenticatedSession,
        request: PullRequest,
    ) -> AppResult<PullResponse> {
        let (actor, _permit) = self.authorize(session, AccessKind::Read)?;
        let request = validation::pull(request)?;
        repository::pull(&self.pool, &actor, request).await
    }
}
