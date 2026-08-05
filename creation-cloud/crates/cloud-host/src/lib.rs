//! Account-owned host metadata, opaque secret storage, and explicit manual sync.

mod actor;
mod repository;
mod router;
mod service;
mod types;
mod validation;

pub use router::{host_router, management_router, router, sync_router};
pub use service::Service;
pub use types::{
    AdminSyncDirection, AdminSyncRecord, HostChange, HostConflictView, HostMetadataInput,
    HostOperation, HostStatus, HostView, LocalDecision, PullAckRequest, PullDecision,
    PullHostRecord, PullRequest, PullResponse, PushOutcome, PushRequest, RekeyHostCandidate,
    RekeyHostRevision, RekeySyncRequest, RekeySyncResponse, RemoteResolution, ResetConfirmation,
    ResetSyncRequest, ResetSyncResponse, ResolveConflictOutcome, ResolveConflictRequest,
    SyncStateView,
};

#[cfg(test)]
mod tests;
