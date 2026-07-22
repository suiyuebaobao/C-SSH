//! 读取当前账号的未解决同步冲突列表。

use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{Service, SyncConflict, limiter::AccessKind, repository};

impl Service {
    pub async fn list_conflicts(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<SyncConflict>> {
        let (actor, _permit) = self.authorize(session, AccessKind::Read)?;
        repository::list_conflicts(&self.pool, &actor, page.normalized()).await
    }
}
