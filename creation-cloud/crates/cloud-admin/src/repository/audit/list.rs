//! 按时间倒序分页读取不可变审计事件。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use sqlx::FromRow;

use crate::{AuditEvent, model::AuditRow, repository::map_read_error};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

pub(crate) async fn execute(pool: &PgPool, query: PageQuery) -> AppResult<Page<AuditEvent>> {
    let query = query.normalized();
    let count = sqlx::query_as::<_, CountRow>("SELECT count(*)::BIGINT AS total FROM audit_events")
        .fetch_one(pool)
        .await
        .map_err(map_read_error)?;
    let rows = sqlx::query_as::<_, AuditRow>(
        r#"
        SELECT id, actor_account_id, action, resource_kind, resource_id,
               outcome, request_id, details, created_at
        FROM audit_events
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(i64::from(query.size))
    .bind(query.offset())
    .fetch_all(pool)
    .await
    .map_err(map_read_error)?;
    let items = rows
        .into_iter()
        .map(AuditEvent::try_from)
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Page {
        items,
        page: query.page,
        size: query.size,
        total: count.total,
    })
}
