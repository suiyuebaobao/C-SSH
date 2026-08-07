//! 保存账号主机与 AI provider 账号的不透明密文并提供显式手动同步。

mod actor;
mod repository;
mod router;
mod service;
mod types;
mod validation;

pub use router::{host_router, management_router, router, sync_router};
pub use service::Service;
pub use types::{
    AdminSyncDirection, AdminSyncRecord, AiProviderChange, AiProviderOperation,
    AiProviderPayloadInput, HostChange, HostMetadataInput, HostOperation, HostStatus, HostView,
    LocalDecision, PullAckRequest, PullAiProviderRecord, PullDecision, PullHostRecord, PullMode,
    PullRequest, PullResponse, PushOutcome, PushRequest, RekeyResourceCandidate, RekeySyncRequest,
    RekeySyncResponse, ResetConfirmation, ResetSyncRequest, ResetSyncResponse, ResourceKind,
    ResourceRevision, SyncGenerationTransition, SyncStateView,
};

#[cfg(test)]
mod tests;
