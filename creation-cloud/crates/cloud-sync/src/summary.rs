//! 定义控制台可读取、且不包含同步正文的账号级摘要。

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct AccountSyncSummary {
    pub current_revision: i64,
    pub active_record_count: i64,
    pub tombstone_count: i64,
    pub unresolved_conflict_count: i64,
    pub recent_records: Vec<SyncRecordSummary>,
    pub recent_conflicts: Vec<SyncConflictSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncRecordSummary {
    pub namespace: String,
    pub key: String,
    pub revision: i64,
    pub deleted: bool,
    pub source_device_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncConflictSummary {
    pub id: Uuid,
    pub base_revision: i64,
    pub current_revision: i64,
    pub source_device_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
