//! 校验解决请求并调用仓储在最新修订上原子解决指定冲突。

use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{
    ResolveConflictOutcome, ResolveConflictRequest, Service, fingerprint, limiter::AccessKind,
    repository, validation,
};

impl Service {
    pub async fn resolve_conflict(
        &self,
        session: &AuthenticatedSession,
        conflict_id: Uuid,
        request: ResolveConflictRequest,
    ) -> AppResult<ResolveConflictOutcome> {
        let (actor, _permit) = self.authorize(session, AccessKind::Write)?;
        validation::conflict_id(conflict_id)?;
        validation::resolve(&request)?;
        let fingerprint = fingerprint::json(&request, "无法计算冲突解决 mutation 指纹")?;
        repository::resolve_conflict(&self.pool, &actor, conflict_id, &request, &fingerprint).await
    }
}
