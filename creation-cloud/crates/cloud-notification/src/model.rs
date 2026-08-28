//! 定义账号通知、分页游标、管理输入与已读回执的公开线协议。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    AccountSecurity,
    Sync,
}

impl NotificationKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::AccountSecurity => "account_security",
            Self::Sync => "sync",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Normal,
    Important,
    Critical,
}

impl NotificationPriority {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Important => "important",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountNotification {
    pub id: Uuid,
    pub revision: i64,
    pub kind: NotificationKind,
    pub priority: NotificationPriority,
    pub code: String,
    pub resource_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub action: String,
    pub read_state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NotificationListResponse {
    pub items: Vec<AccountNotification>,
    pub next_cursor: Option<String>,
    pub unread_count: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateNotificationInput {
    pub account_id: Uuid,
    pub kind: NotificationKind,
    pub priority: NotificationPriority,
    pub code: String,
    pub resource_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AdminNotification {
    pub id: Uuid,
    pub account_id: Uuid,
    pub revision: i64,
    pub kind: NotificationKind,
    pub priority: NotificationPriority,
    pub code: String,
    pub resource_id: Option<Uuid>,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptInput {
    pub revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct NotificationReceipt {
    pub notification_id: Uuid,
    pub revision: i64,
    pub read_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct NotificationRow {
    pub id: Uuid,
    pub account_id: Uuid,
    pub revision: i64,
    pub kind: String,
    pub priority: String,
    pub code: String,
    pub resource_id: Option<Uuid>,
    pub published_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub read_at: Option<DateTime<Utc>>,
}
