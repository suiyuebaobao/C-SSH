//! 提供非敏感偏好的白名单同步、修订拉取、冲突与墓碑能力。
//! 本 crate 不接收主机、SSH、Token、终端或 AI 内容，也不依赖其它业务域。

mod actor;
mod fingerprint;
mod handler;
mod limiter;
mod model;
mod repository;
mod router;
mod service;
mod summary;
mod types;
mod use_case;
mod validation;

pub use model::retention::RetentionReport;
pub use router::router;
pub use service::Service;
pub use summary::{AccountSyncSummary, SyncConflictSummary, SyncRecordSummary};
pub use types::{
    ConflictResolution, PullMode, PullRequest, PullResponse, PushOutcome, PushRequest,
    ResolveConflictOutcome, ResolveConflictRequest, SyncChange, SyncConflict, SyncOperation,
    SyncRecord,
};

#[cfg(test)]
mod tests;
