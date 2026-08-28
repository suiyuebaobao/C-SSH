//! 汇总来源、公开清单和内部下载目标模型。

mod aggregation;
mod history;
mod inspection;
mod installed_identity;
mod policy;
mod public;
mod source;
mod target;
mod tauri_update;
mod update;

pub use aggregation::DownloadAggregationReport;
pub(crate) use aggregation::{DownloadAggregateBucket, DownloadAudience, PendingDownloadEvent};
pub use history::DownloadHistoryItem;
pub(crate) use inspection::{AssetInspectionObservation, FileInspection, PublishedLocalAsset};
pub use inspection::{AssetInspectionStatus, PublishedAssetInspectionReport};
pub use installed_identity::{
    InstalledIdentityEntryInput, RecordInstalledIdentitiesInput, RecordInstalledIdentitiesResult,
};
pub use policy::{
    AdminUpdatePolicySnapshot, PublishUpdatePolicyInput, PublishedUpdatePolicy,
    SaveUpdatePolicyDraftInput, UpdatePolicyDraft, UpdatePolicyTargetRelease,
};
pub(crate) use policy::{
    ForcedIdentityRow, PolicyAssetRow, PolicyTargetRow, PublishedUpdatePolicyRow,
    UpdatePolicyDraftRow,
};
pub(crate) use public::PublicCatalogRow;
pub use public::{PublicAsset, PublicRelease, PublicSource};
pub(crate) use source::SourceRow;
pub use source::{CreateSourceInput, ReleaseSource, SourceKind, UpdateSourceInput};
pub(crate) use target::{AssetRecord, DownloadTarget, LockedAssetRecord};
pub(crate) use tauri_update::{TauriPlatformUpdate, TauriUpdateQuery, TauriUpdateResponse};
pub use update::{
    LatestUpdate, UpdateAsset, UpdateCheckQuery, UpdateCheckResponse, UpdateIdentityStatus,
    UpdateSource,
};
