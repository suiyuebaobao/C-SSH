//! 以期望版本原子更新状态或把正文替换为固定墓碑，永不提供恢复与物理删除。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{FeedbackStatus, model::FeedbackRow};

use super::{error, semantic_audit::AuditContext};

pub(crate) struct StatusMutation<'a> {
    pub(crate) audit: AuditContext<'a>,
    pub(crate) expected_version: i64,
    pub(crate) current: FeedbackStatus,
    pub(crate) target: FeedbackStatus,
}

pub(crate) struct RedactionMutation<'a> {
    pub(crate) audit: AuditContext<'a>,
    pub(crate) expected_version: i64,
}

pub(crate) const REDACTED_TITLE: &str = "[已由管理员安全脱敏]";
pub(crate) const REDACTED_DESCRIPTION: &str = "[反馈正文已由管理员执行不可逆安全脱敏]";
pub(crate) const STATUS_UPDATE_SQL: &str = r#"
    WITH updated AS (
        UPDATE feedback_submissions
        SET status = $3, version = version + 1, updated_at = now()
        WHERE id = $1 AND version = $2
        RETURNING id, account_id, request_id, category, platform, app_version,
                  title, description, status, version, redacted_at, redaction_reason,
                  created_at, updated_at
    ), audited AS (
        SELECT record_feedback_semantic_audit(
            $4, $5, 'feedback.status_changed', $1, 'success', $6,
            jsonb_build_object(
                'feedback_id', $1::TEXT,
                'from_status', $7::TEXT,
                'to_status', $3::TEXT,
                'reason', $8::TEXT,
                'failure_code', to_jsonb(NULL::TEXT)
            )
        )
        FROM updated
    )
    SELECT updated.id, updated.account_id, updated.request_id, updated.category,
           updated.platform, updated.app_version, updated.title, updated.description,
           updated.status, updated.version, updated.redacted_at, updated.redaction_reason,
           updated.created_at, updated.updated_at
    FROM updated
    JOIN audited ON TRUE
"#;
pub(crate) const REDACT_SQL: &str = r#"
    WITH updated AS (
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
    ), audited AS (
        SELECT record_feedback_semantic_audit(
            $7, $6, 'feedback.redacted', $1, 'success', $8,
            jsonb_build_object(
                'feedback_id', $1::TEXT,
                'reason_summary', $5::TEXT,
                'failure_code', to_jsonb(NULL::TEXT)
            )
        )
        FROM updated
    )
    SELECT updated.id, updated.account_id, updated.request_id, updated.category,
           updated.platform, updated.app_version, updated.title, updated.description,
           updated.status, updated.version, updated.redacted_at, updated.redaction_reason,
           updated.created_at, updated.updated_at
    FROM updated
    JOIN audited ON TRUE
"#;

pub(crate) async fn status(
    connection: &mut PgConnection,
    input: &StatusMutation<'_>,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(STATUS_UPDATE_SQL)
        .bind(input.audit.feedback_id)
        .bind(input.expected_version)
        .bind(input.target.as_str())
        .bind(Uuid::now_v7())
        .bind(input.audit.actor_id)
        .bind(input.audit.request_id)
        .bind(input.current.as_str())
        .bind(input.audit.reason)
        .fetch_optional(connection)
        .await
        .map_err(error::semantic_audit)
}

pub(crate) async fn redact(
    connection: &mut PgConnection,
    input: &RedactionMutation<'_>,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(REDACT_SQL)
        .bind(input.audit.feedback_id)
        .bind(input.expected_version)
        .bind(REDACTED_TITLE)
        .bind(REDACTED_DESCRIPTION)
        .bind(input.audit.reason)
        .bind(input.audit.actor_id)
        .bind(Uuid::now_v7())
        .bind(input.audit.request_id)
        .fetch_optional(connection)
        .await
        .map_err(error::semantic_audit)
}
