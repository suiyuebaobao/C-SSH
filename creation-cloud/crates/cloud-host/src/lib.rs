//! Account-owned host metadata, opaque secret storage, and explicit manual sync.

mod actor;
mod repository;
mod router;
mod service;
mod types;
mod validation;

pub use router::{device_router, host_router, management_router, router, sync_router};
pub use service::Service;
pub use types::{
    HostChange, HostConflictView, HostDownloadAllowlist, HostMetadataInput, HostOperation,
    HostStatus, HostView, LocalDecision, PullAckRequest, PullDecision, PullHostRecord, PullRequest,
    PullResponse, PushOutcome, PushRequest, RemoteResolution, ReplaceAllowlistRequest,
    ResolveConflictOutcome, ResolveConflictRequest,
};

#[cfg(test)]
mod tests;
