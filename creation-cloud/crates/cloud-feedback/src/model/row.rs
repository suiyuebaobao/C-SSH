//! 定义仅在 repository 与响应映射之间传递的 PostgreSQL 行结构。

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow)]
pub(crate) struct FeedbackRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub request_id: String,
    pub category: String,
    pub platform: String,
    pub app_version: Option<String>,
    pub title: String,
    pub description: String,
    pub status: String,
    pub version: i64,
    pub redacted_at: Option<DateTime<Utc>>,
    pub redaction_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct FeedbackSummaryRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category: String,
    pub platform: String,
    pub status: String,
    pub version: i64,
    pub redacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
