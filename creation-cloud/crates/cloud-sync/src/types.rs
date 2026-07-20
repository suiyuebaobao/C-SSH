//! 定义同步 API 的稳定输入、输出和冲突表示。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOperation {
    Upsert,
    Delete,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SyncChange {
    pub namespace: String,
    pub key: String,
    pub operation: SyncOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushRequest {
    pub base_revision: i64,
    pub client_mutation_id: Uuid,
    pub changes: Vec<SyncChange>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PushOutcome {
    Applied {
        revision: i64,
        idempotent: bool,
    },
    Conflict {
        conflict: SyncConflict,
        idempotent: bool,
    },
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct PullRequest {
    #[serde(default)]
    pub since_revision: i64,
    #[serde(default = "default_pull_limit")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PullResponse {
    pub records: Vec<SyncRecord>,
    pub next_revision: i64,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncRecord {
    pub namespace: String,
    pub key: String,
    pub revision: i64,
    pub deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncConflict {
    pub id: Uuid,
    pub client_mutation_id: Uuid,
    pub base_revision: i64,
    pub current_revision: i64,
    pub attempted_changes: Vec<SyncChange>,
    pub created_at: DateTime<Utc>,
}

const fn default_pull_limit() -> u32 {
    100
}
