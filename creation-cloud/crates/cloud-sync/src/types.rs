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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    KeepRemote,
    ApplyChanges,
}

impl ConflictResolution {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::KeepRemote => "keep_remote",
            Self::ApplyChanges => "apply_changes",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "resolution", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolveConflictRequest {
    KeepRemote {
        resolution_mutation_id: Uuid,
    },
    ApplyChanges {
        resolution_mutation_id: Uuid,
        changes: Vec<SyncChange>,
    },
}

impl ResolveConflictRequest {
    pub(crate) const fn resolution_mutation_id(&self) -> Uuid {
        match self {
            Self::KeepRemote {
                resolution_mutation_id,
            }
            | Self::ApplyChanges {
                resolution_mutation_id,
                ..
            } => *resolution_mutation_id,
        }
    }

    pub(crate) const fn resolution(&self) -> ConflictResolution {
        match self {
            Self::KeepRemote { .. } => ConflictResolution::KeepRemote,
            Self::ApplyChanges { .. } => ConflictResolution::ApplyChanges,
        }
    }

    pub(crate) fn changes(&self) -> Option<&[SyncChange]> {
        match self {
            Self::KeepRemote { .. } => None,
            Self::ApplyChanges { changes, .. } => Some(changes),
        }
    }
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

#[derive(Clone, Debug, Serialize)]
pub struct ResolveConflictOutcome {
    pub conflict_id: Uuid,
    pub resolution_mutation_id: Uuid,
    pub resolution: ConflictResolution,
    pub revision: i64,
    pub idempotent: bool,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PullMode {
    #[default]
    Incremental,
    Full,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct PullRequest {
    #[serde(default)]
    pub since_revision: i64,
    #[serde(default)]
    pub mode: PullMode,
    #[serde(default)]
    pub snapshot_revision: Option<i64>,
    #[serde(default = "default_pull_limit")]
    pub limit: u32,
}

impl Default for PullRequest {
    fn default() -> Self {
        Self {
            since_revision: 0,
            mode: PullMode::Incremental,
            snapshot_revision: None,
            limit: default_pull_limit(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PullResponse {
    pub records: Vec<SyncRecord>,
    pub next_revision: i64,
    pub has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_revision: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncRecord {
    pub namespace: String,
    pub key: String,
    pub revision: i64,
    pub deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    pub source_device_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SyncConflict {
    pub id: Uuid,
    pub client_mutation_id: Uuid,
    pub base_revision: i64,
    pub current_revision: i64,
    pub attempted_changes: Vec<SyncChange>,
    pub source_device_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

const fn default_pull_limit() -> u32 {
    100
}
