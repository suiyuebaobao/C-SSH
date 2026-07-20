//! 分页读取管理端审计时间线。

use cloud_domain::{AdminActor, AppResult, Page, PageQuery};

use crate::{AuditEvent, Service, repository};

impl Service {
    pub async fn list_audit_events(
        &self,
        actor: &AdminActor,
        query: PageQuery,
    ) -> AppResult<Page<AuditEvent>> {
        crate::validation::admin_actor(actor)?;
        repository::audit::list::execute(&self.pool, crate::validation::page(query)).await
    }
}
