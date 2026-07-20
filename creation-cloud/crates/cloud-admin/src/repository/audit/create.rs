//! 插入一条不可变且已经过脱敏校验的审计事件。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AuditEvent, model::AuditInsert, model::AuditRow, repository::map_write_error};

pub(crate) async fn execute(pool: &PgPool, input: &AuditInsert) -> AppResult<AuditEvent> {
    let row = sqlx::query_as::<_, AuditRow>(
        r#"
        INSERT INTO audit_events (
            id, actor_account_id, action, resource_kind, resource_id,
            outcome, request_id, details
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, actor_account_id, action, resource_kind, resource_id,
                  outcome, request_id, details, created_at
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(input.actor_account_id)
    .bind(&input.action)
    .bind(&input.resource_kind)
    .bind(input.resource_id.as_deref())
    .bind(input.outcome.as_str())
    .bind(input.request_id.as_deref())
    .bind(&input.details)
    .fetch_one(pool)
    .await
    .map_err(map_write_error)?;
    AuditEvent::try_from(row)
}
