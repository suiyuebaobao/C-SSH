//! 定义反馈域面向管理总览的最小状态计数，不携带任何提交正文或账号身份。

use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct FeedbackOverview {
    pub total_feedback: i64,
    pub new_feedback: i64,
    pub triaged_feedback: i64,
    pub in_progress_feedback: i64,
    pub resolved_feedback: i64,
    pub closed_feedback: i64,
}
