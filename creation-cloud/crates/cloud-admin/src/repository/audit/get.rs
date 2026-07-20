//! 按内部标识读取单条审计事件。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AuditEvent, model::AuditRow, repository::map_read_error};

pub(crate) async fn execute(pool: &PgPool, event_id: Uuid) -> AppResult<AuditEvent> {
    let row = sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT id, actor_account_id, action, resource_kind, resource_id,
               outcome, request_id, details, created_at
        FROM audit_events
        WHERE id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(map_read_error)?
    .ok_or_else(|| AppError::NotFound("审计事件不存在".into()))?;
    AuditEvent::try_from(row)
}
