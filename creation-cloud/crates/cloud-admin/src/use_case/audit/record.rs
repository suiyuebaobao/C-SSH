//! 将服务端请求上下文收敛为无正文、无邮箱且 actor 不可伪造的审计事件。

use cloud_domain::{AdminActor, AppResult};
use serde_json::json;

use crate::{AuditOutcome, Service, model::AuditInsert, repository, validation};

#[derive(Clone, Debug)]
pub(crate) struct HttpAuditRecord {
    pub method: String,
    pub path: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub status: u16,
    pub request_id: String,
}

impl Service {
    pub(crate) async fn record_http_request(
        &self,
        actor: &AdminActor,
        record: HttpAuditRecord,
    ) -> AppResult<()> {
        let actor_account_id = validation::admin_actor(actor)?;
        let input = normalize(AuditInsert {
            actor_account_id: Some(actor_account_id),
            action: format!("http.admin.{}", record.method.to_ascii_lowercase()),
            resource_kind: record.resource_kind,
            resource_id: record.resource_id,
            outcome: if record.status < 400 {
                AuditOutcome::Success
            } else {
                AuditOutcome::Failure
            },
            request_id: Some(record.request_id),
            details: json!({
                "method": record.method,
                "path": record.path,
                "status": record.status,
            }),
        })?;
        repository::audit::create::execute(&self.pool, &input).await?;
        Ok(())
    }
}

pub(crate) fn normalize(input: AuditInsert) -> AppResult<AuditInsert> {
    Ok(AuditInsert {
        actor_account_id: validation::optional_id(input.actor_account_id, "操作者标识")?,
        action: validation::required_code(&input.action, "审计动作", 128)?,
        resource_kind: validation::required_code(&input.resource_kind, "资源类型", 64)?,
        resource_id: validation::optional_text(input.resource_id.as_deref(), "资源标识", 200)?,
        outcome: input.outcome,
        request_id: validation::optional_text(input.request_id.as_deref(), "请求标识", 128)?,
        details: validation::details(input.details)?,
    })
}
