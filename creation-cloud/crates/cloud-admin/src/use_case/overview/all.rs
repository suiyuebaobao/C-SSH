//! 并发组合四类互不依赖的只读管理概览。

use cloud_domain::{AdminActor, AppResult};

use crate::{AdminOverview, Service};

impl Service {
    pub async fn overview(&self, actor: &AdminActor) -> AppResult<AdminOverview> {
        crate::validation::admin_actor(actor)?;
        let (users, devices, releases, audit) = tokio::try_join!(
            self.user_overview(actor),
            self.device_overview(actor),
            self.release_overview(actor),
            self.security_audit_overview(actor)
        )?;
        Ok(AdminOverview {
            users,
            devices,
            releases,
            audit,
        })
    }
}
