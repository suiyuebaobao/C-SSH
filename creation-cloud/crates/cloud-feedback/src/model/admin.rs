//! 定义管理列表最小摘要与显式单查详情，列表绝不携带标题或正文。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use serde::Serialize;
use uuid::Uuid;

use super::{FeedbackCategory, FeedbackPlatform, FeedbackRow, FeedbackStatus, FeedbackSummaryRow};

#[derive(Clone, Debug, Serialize)]
pub struct AdminFeedbackSummary {
    pub id: Uuid,
    pub masked_account_id: String,
    pub category: FeedbackCategory,
    pub platform: FeedbackPlatform,
    pub status: FeedbackStatus,
    pub version: i64,
    pub redacted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminFeedbackDetail {
    pub id: Uuid,
    pub masked_account_id: String,
    pub request_id: String,
    pub category: FeedbackCategory,
    pub platform: FeedbackPlatform,
    pub app_version: Option<String>,
    pub title: String,
    pub description: String,
    pub status: FeedbackStatus,
    pub version: i64,
    pub redacted: bool,
    pub redaction_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<FeedbackSummaryRow> for AdminFeedbackSummary {
    type Error = cloud_domain::AppError;

    fn try_from(row: FeedbackSummaryRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            masked_account_id: mask_account(row.account_id),
            category: FeedbackCategory::try_from(row.category.as_str())?,
            platform: FeedbackPlatform::try_from(row.platform.as_str())?,
            status: FeedbackStatus::try_from(row.status.as_str())?,
            version: row.version,
            redacted: row.redacted_at.is_some(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<FeedbackRow> for AdminFeedbackDetail {
    type Error = cloud_domain::AppError;

    fn try_from(row: FeedbackRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            masked_account_id: mask_account(row.account_id),
            request_id: row.request_id,
            category: FeedbackCategory::try_from(row.category.as_str())?,
            platform: FeedbackPlatform::try_from(row.platform.as_str())?,
            app_version: row.app_version,
            title: row.title,
            description: row.description,
            status: FeedbackStatus::try_from(row.status.as_str())?,
            version: row.version,
            redacted: row.redacted_at.is_some(),
            redaction_reason: row.redaction_reason,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn mask_account(account_id: Uuid) -> String {
    let value = account_id.simple().to_string();
    format!("{}************************", &value[..8])
}
