//! 定义用户创建、管理员状态迁移与不可逆脱敏的受限 JSON 输入。

use serde::Deserialize;

use super::{FeedbackCategory, FeedbackPlatform, FeedbackStatus};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateFeedbackInput {
    pub category: FeedbackCategory,
    pub platform: FeedbackPlatform,
    pub app_version: Option<String>,
    pub title: String,
    pub description: String,
    pub redaction_confirmed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateFeedbackStatusInput {
    pub status: FeedbackStatus,
    pub expected_version: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactFeedbackInput {
    pub expected_version: i64,
    pub reason: String,
}
