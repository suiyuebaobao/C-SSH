//! 在反馈事务内调用数据库受约束入口，记录不含正文与邮箱的失败语义审计。

use cloud_domain::AppResult;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::FeedbackStatus;

use super::error;

#[derive(Clone, Copy)]
pub(crate) struct AuditContext<'a> {
    pub(crate) actor_id: Uuid,
    pub(crate) feedback_id: Uuid,
    pub(crate) request_id: &'a str,
    pub(crate) reason: &'a str,
}

pub(crate) struct StatusFailure<'a> {
    pub(crate) audit: AuditContext<'a>,
    pub(crate) from_status: Option<FeedbackStatus>,
    pub(crate) to_status: FeedbackStatus,
    pub(crate) failure_code: &'a str,
}

pub(crate) struct RedactionFailure<'a> {
    pub(crate) audit: AuditContext<'a>,
    pub(crate) failure_code: &'a str,
}

pub(crate) const STATUS_FAILURE_SQL: &str = r#"
    SELECT record_feedback_semantic_audit(
        $1, $2, 'feedback.status_changed', $3, 'failure', $4,
        jsonb_build_object(
            'feedback_id', $3::TEXT,
            'from_status', $5::TEXT,
            'to_status', $6::TEXT,
            'reason', $7::TEXT,
            'failure_code', $8::TEXT
        )
    )
"#;

pub(crate) const REDACTION_FAILURE_SQL: &str = r#"
    SELECT record_feedback_semantic_audit(
        $1, $2, 'feedback.redacted', $3, 'failure', $4,
        jsonb_build_object(
            'feedback_id', $3::TEXT,
            'reason_summary', $5::TEXT,
            'failure_code', $6::TEXT
        )
    )
"#;

pub(crate) async fn status_failure(
    connection: &mut PgConnection,
    input: &StatusFailure<'_>,
) -> AppResult<()> {
    sqlx::query(STATUS_FAILURE_SQL)
        .bind(Uuid::now_v7())
        .bind(input.audit.actor_id)
        .bind(input.audit.feedback_id)
        .bind(input.audit.request_id)
        .bind(input.from_status.map(FeedbackStatus::as_str))
        .bind(input.to_status.as_str())
        .bind(input.audit.reason)
        .bind(input.failure_code)
        .execute(connection)
        .await
        .map_err(error::semantic_audit)?;
    Ok(())
}

pub(crate) async fn redaction_failure(
    connection: &mut PgConnection,
    input: &RedactionFailure<'_>,
) -> AppResult<()> {
    sqlx::query(REDACTION_FAILURE_SQL)
        .bind(Uuid::now_v7())
        .bind(input.audit.actor_id)
        .bind(input.audit.feedback_id)
        .bind(input.audit.request_id)
        .bind(input.audit.reason)
        .bind(input.failure_code)
        .execute(connection)
        .await
        .map_err(error::semantic_audit)?;
    Ok(())
}
