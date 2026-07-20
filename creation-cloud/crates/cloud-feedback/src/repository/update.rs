//! 以期望版本原子更新状态或把正文替换为固定墓碑，永不提供恢复与物理删除。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{FeedbackStatus, model::FeedbackRow};

use super::error;

pub(crate) const REDACTED_TITLE: &str = "[已由管理员安全脱敏]";
pub(crate) const REDACTED_DESCRIPTION: &str = "[反馈正文已由管理员执行不可逆安全脱敏]";
pub(crate) const STATUS_UPDATE_SQL: &str = r#"
    UPDATE feedback_submissions
    SET status = $3, version = version + 1, updated_at = now()
    WHERE id = $1 AND version = $2
    RETURNING id, account_id, request_id, category, platform, app_version,
              title, description, status, version, redacted_at, redaction_reason,
              created_at, updated_at
"#;
pub(crate) const REDACT_SQL: &str = r#"
    UPDATE feedback_submissions
    SET title = $3,
        description = $4,
        status = 'closed',
        version = version + 1,
        redacted_at = now(),
        redacted_by = $6,
        redaction_reason = $5,
        updated_at = now()
    WHERE id = $1 AND version = $2 AND redacted_at IS NULL
    RETURNING id, account_id, request_id, category, platform, app_version,
              title, description, status, version, redacted_at, redaction_reason,
              created_at, updated_at
"#;

pub(crate) async fn status(
    pool: &PgPool,
    id: Uuid,
    expected_version: i64,
    target: FeedbackStatus,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(STATUS_UPDATE_SQL)
        .bind(id)
        .bind(expected_version)
        .bind(target.as_str())
        .fetch_optional(pool)
        .await
        .map_err(error::write)
}

pub(crate) async fn redact(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    expected_version: i64,
    reason: &str,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(REDACT_SQL)
        .bind(id)
        .bind(expected_version)
        .bind(REDACTED_TITLE)
        .bind(REDACTED_DESCRIPTION)
        .bind(reason)
        .bind(actor_id)
        .fetch_optional(pool)
        .await
        .map_err(error::write)
}
