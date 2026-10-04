//! 以单条聚合查询读取反馈状态分布，避免管理装配层跨域读取反馈表。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::FeedbackOverview;

use super::error;

pub(crate) const OVERVIEW_SQL: &str = r#"
    SELECT
        COUNT(*)::BIGINT AS total_feedback,
        COUNT(*) FILTER (WHERE status = 'new')::BIGINT AS new_feedback,
        COUNT(*) FILTER (WHERE status = 'triaged')::BIGINT AS triaged_feedback,
        COUNT(*) FILTER (WHERE status = 'in_progress')::BIGINT AS in_progress_feedback,
        COUNT(*) FILTER (WHERE status = 'resolved')::BIGINT AS resolved_feedback,
        COUNT(*) FILTER (WHERE status = 'closed')::BIGINT AS closed_feedback
    FROM feedback_submissions
"#;

pub(crate) async fn execute(pool: &PgPool) -> AppResult<FeedbackOverview> {
    sqlx::query_as::<_, FeedbackOverview>(OVERVIEW_SQL)
        .fetch_one(pool)
        .await
        .map_err(error::read)
}
