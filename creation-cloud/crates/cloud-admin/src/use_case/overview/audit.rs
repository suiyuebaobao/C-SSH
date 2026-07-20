//! 读取审计结果管理概览。

use cloud_domain::{AdminActor, AppResult};

use crate::{SecurityAuditOverview, Service, repository};

impl Service {
    pub async fn security_audit_overview(
        &self,
        actor: &AdminActor,
    ) -> AppResult<SecurityAuditOverview> {
        crate::validation::admin_actor(actor)?;
        repository::overview::audit::execute(&self.pool).await
    }
}
