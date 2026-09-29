//! 保存账号主机与 AI provider 账号的不透明密文并提供显式手动同步。

mod actor;
mod fingerprint;
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
    AiProviderPayloadInput, CURRENT_SYNC_CONTRACT_VERSION, ChangeDataProtectionRequest,
    DataProtectionEnvelopeInput, DataProtectionEnvelopeView, DataProtectionMigrationReceipt,
    DataProtectionMutationResponse, DataProtectionView, HostChange, HostMetadataInput,
    HostMetadataMigrationCandidate, HostMetadataMigrationPreviewRecord,
    HostMetadataMigrationPreviewRequest, HostMetadataMigrationPreviewResponse,
    HostMetadataMigrationReceipt, HostMetadataMigrationRequest, HostOperation, HostStatus,
    HostView, LEGACY_SYNC_CONTRACT_VERSION, LegacyPullCursor, LegacyPullHostRecord,
    LegacyPullRequest, LegacyPullResponse, LocalDecision, MigrateDataProtectionRequest,
    ProtectionResetChallengeRequest, ProtectionResetChallengeResponse, ProxyProfileChange,
    ProxyProfileOperation, ProxyProfilePayloadInput, PullAckRequest, PullAiProviderRecord,
    PullDecision, PullHostRecord, PullMode, PullProxyProfileRecord, PullPurpose, PullRequest,
    PullResponse, PushOutcome, PushReceipt, PushRequest, RekeyResourceCandidate, RekeySyncRequest,
    RekeySyncResponse, RequiredNullableCiphertext, ResetAuthorization, ResetConfirmation,
    ResetSyncRequest, ResetSyncResponse, ResourceKind, ResourceRevision,
    SetupDataProtectionRequest, SyncGenerationTransition, SyncStateView,
    VerifyProtectionResetChallengeRequest, VerifyProtectionResetChallengeResponse,
};

#[cfg(test)]
mod tests;
