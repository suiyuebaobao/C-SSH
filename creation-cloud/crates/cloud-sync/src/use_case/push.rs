//! 校验完整 mutation，并调用仓储执行幂等、冲突或原子写入。

use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{
    PushOutcome, PushRequest, Service, fingerprint, limiter::AccessKind, repository, validation,
};

impl Service {
    pub async fn push(
        &self,
        session: &AuthenticatedSession,
        request: PushRequest,
    ) -> AppResult<PushOutcome> {
        let (actor, _permit) = self.authorize(session, AccessKind::Write)?;
        validation::push(&request)?;
        let fingerprint = fingerprint::json(&request, "无法计算同步 mutation 指纹")?;
        repository::push(&self.pool, &actor, &request, &fingerprint).await
    }
}
