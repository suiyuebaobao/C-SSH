//! 旧durable push的只读receipt；只返回身份、摘要方案和revision结果。

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::ResourceRevision;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PushReceipt {
    pub status: String,
    pub mutation_id: Uuid,
    pub source_device_id: Uuid,
    pub request_sync_generation: i64,
    pub request_protection_epoch: i64,
    pub request_protection_revision: i64,
    pub result_revision: i64,
    pub changed_count: i32,
    pub request_hash: String,
    pub request_hash_scheme: String,
    pub revisions: Vec<ResourceRevision>,
    pub created_at: DateTime<Utc>,
}
