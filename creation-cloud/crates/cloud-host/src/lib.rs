//! 保存账号主机与 AI provider 账号的不透明密文并提供显式手动同步。

mod actor;
mod protection_mailer;
mod repository;
mod router;
mod service;
mod types;
mod validation;

pub use protection_mailer::{ProtectionResetMailer, ProtectionResetMailerFuture};
pub use router::{host_router, management_router, router, sync_router};
pub use service::Service;
pub use types::{
    AdminSyncDirection, AdminSyncRecord, AiProviderChange, AiProviderOperation,
    AiProviderPayloadInput, ChangeDataProtectionRequest, DataProtectionEnvelopeInput,
    DataProtectionEnvelopeView, DataProtectionMutationResponse, DataProtectionView, HostChange,
    HostMetadataInput, HostOperation, HostStatus, HostView, LegacyPullCursor, LegacyPullRequest,
    LegacyPullResponse, LocalDecision, MigrateDataProtectionRequest,
    ProtectionResetChallengeRequest, ProtectionResetChallengeResponse, PullAckRequest,
    PullAiProviderRecord, PullDecision, PullHostRecord, PullMode, PullPurpose, PullRequest,
    PullResponse, PushOutcome, PushRequest, RekeyResourceCandidate, RekeySyncRequest,
    RekeySyncResponse, ResetAuthorization, ResetConfirmation, ResetSyncRequest, ResetSyncResponse,
    ResourceKind, ResourceRevision, SetupDataProtectionRequest, SyncGenerationTransition,
    SyncStateView, VerifyProtectionResetChallengeRequest, VerifyProtectionResetChallengeResponse,
};

#[cfg(test)]
mod tests;
