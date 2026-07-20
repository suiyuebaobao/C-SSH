//! 定义不可变审计事件、结果枚举和仅供服务端构造的写入模型。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    Success,
    Failure,
}

impl AuditOutcome {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

impl TryFrom<&str> for AuditOutcome {
    type Error = AppError;

    fn try_from(value: &str) -> AppResult<Self> {
        match value {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            _ => Err(AppError::Internal("数据库中的审计结果无效".into())),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub outcome: AuditOutcome,
    pub request_id: Option<String>,
    pub details: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub(crate) struct AuditRow {
    pub id: Uuid,
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub outcome: String,
    pub request_id: Option<String>,
    pub details: Value,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<AuditRow> for AuditEvent {
    type Error = AppError;

    fn try_from(row: AuditRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id,
            actor_account_id: row.actor_account_id,
            action: row.action,
            resource_kind: row.resource_kind,
            resource_id: row.resource_id,
            outcome: AuditOutcome::try_from(row.outcome.as_str())?,
            request_id: row.request_id,
            details: row.details,
            created_at: row.created_at,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AuditInsert {
    pub actor_account_id: Option<Uuid>,
    pub action: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub outcome: AuditOutcome,
    pub request_id: Option<String>,
    pub details: Value,
}
