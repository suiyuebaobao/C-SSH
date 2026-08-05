//! 在用户管理写事务中追加不含账号标识正文的语义审计。

use cloud_domain::{AppResult, current_request_id};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) const INSERT_AUDIT_SQL: &str = r#"
    INSERT INTO audit_events (
        id, actor_account_id, action, resource_kind, resource_id,
        outcome, request_id, details
    )
    VALUES ($1, $2, $3, 'account', $4, 'success', $5, '{}'::jsonb)
"#;

pub(crate) async fn record(
    transaction: &mut Transaction<'_, Postgres>,
    actor_account_id: Uuid,
    target_account_id: Uuid,
    action: &str,
) -> AppResult<()> {
    sqlx::query(INSERT_AUDIT_SQL)
        .bind(Uuid::now_v7())
        .bind(actor_account_id)
        .bind(action)
        .bind(target_account_id.to_string())
        .bind(current_request_id())
        .execute(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    Ok(())
}
