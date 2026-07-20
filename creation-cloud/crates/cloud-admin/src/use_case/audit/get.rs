//! 校验审计标识并读取单条事件。

use cloud_domain::{AdminActor, AppResult};
use uuid::Uuid;

use crate::{AuditEvent, Service, repository, validation};

impl Service {
    pub async fn get_audit_event(
        &self,
        actor: &AdminActor,
        event_id: Uuid,
    ) -> AppResult<AuditEvent> {
        validation::admin_actor(actor)?;
        repository::audit::get::execute(&self.pool, validation::valid_id(event_id, "审计事件标识")?)
            .await
    }
}
