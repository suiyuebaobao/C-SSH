//! 按账号所有权读取单个同步冲突。

use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, SyncConflict, limiter::AccessKind, repository, validation};

impl Service {
    pub async fn get_conflict(
        &self,
        session: &AuthenticatedSession,
        conflict_id: Uuid,
    ) -> AppResult<SyncConflict> {
        let (actor, _permit) = self.authorize(session, AccessKind::Read)?;
        validation::conflict_id(conflict_id)?;
        repository::get_conflict(&self.pool, &actor, conflict_id).await
    }
}
