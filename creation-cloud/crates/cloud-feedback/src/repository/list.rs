//! 分页查询本人完整反馈或管理员最小摘要，管理列表不读取标题与正文列。

use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::model::{FeedbackRow, FeedbackSummaryRow};

use super::error;

pub(crate) const MANAGEMENT_LIST_SQL: &str = "SELECT id, account_id, category, platform, status, version, redacted_at, \
            created_at, updated_at \
     FROM feedback_submissions \
     WHERE ($1::text IS NULL OR status = $1) \
     ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3";
pub(crate) const MANAGEMENT_COUNT_SQL: &str =
    "SELECT COUNT(*) FROM feedback_submissions WHERE ($1::text IS NULL OR status = $1)";
pub(crate) const OWNED_LIST_SQL: &str = r#"
    SELECT id, account_id, request_id, category, platform, app_version,
           title, description, status, version, redacted_at, redaction_reason,
           created_at, updated_at
    FROM feedback_submissions
    WHERE account_id = $1
    ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3
"#;

pub(crate) async fn owned(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<FeedbackRow>, i64)> {
    let items = sqlx::query_as::<_, FeedbackRow>(OWNED_LIST_SQL)
        .bind(account_id)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(error::read)?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM feedback_submissions WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(error::read)?;
    Ok((items, total))
}

pub(crate) async fn management(
    pool: &PgPool,
    page: PageQuery,
    status: Option<crate::FeedbackStatus>,
) -> AppResult<(Vec<FeedbackSummaryRow>, i64)> {
    let status = status.map(crate::FeedbackStatus::as_str);
    let items = sqlx::query_as::<_, FeedbackSummaryRow>(MANAGEMENT_LIST_SQL)
        .bind(status)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(error::read)?;
    let total = sqlx::query_scalar::<_, i64>(MANAGEMENT_COUNT_SQL)
        .bind(status)
        .fetch_one(pool)
        .await
        .map_err(error::read)?;
    Ok((items, total))
}
