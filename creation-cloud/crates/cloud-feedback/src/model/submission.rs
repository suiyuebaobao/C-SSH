//! 将数据库权威行收敛为不暴露账号或内部请求标识的本人反馈响应。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use serde::Serialize;
use uuid::Uuid;

use super::{FeedbackCategory, FeedbackPlatform, FeedbackRow, FeedbackStatus};

#[derive(Clone, Debug, Serialize)]
pub struct FeedbackSubmission {
    pub id: Uuid,
    pub category: FeedbackCategory,
    pub platform: FeedbackPlatform,
    pub app_version: Option<String>,
    pub title: String,
    pub description: String,
    pub status: FeedbackStatus,
    pub version: i64,
    pub redacted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<FeedbackRow> for FeedbackSubmission {
    type Error = cloud_domain::AppError;

    fn try_from(row: FeedbackRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            category: FeedbackCategory::try_from(row.category.as_str())?,
            platform: FeedbackPlatform::try_from(row.platform.as_str())?,
            app_version: row.app_version,
            title: row.title,
            description: row.description,
            status: FeedbackStatus::try_from(row.status.as_str())?,
            version: row.version,
            redacted: row.redacted_at.is_some(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
