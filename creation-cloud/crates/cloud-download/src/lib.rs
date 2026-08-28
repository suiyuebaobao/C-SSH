//! 管理安装资产的多来源元数据与公开下载分发。
//! 常规来源仍只读投影版本；极简后台组合用例通过发布域受控事务入口原子创建资产与来源。

mod authorization;
mod file_verification;
mod handler;
mod limiter;
mod local_file;
mod model;
mod range;
mod readiness;
mod repository;
mod router;
mod service;
mod signature;
mod stored_file_delete;
mod upload_file;
mod use_case;
mod validation;

#[cfg(test)]
mod router_tests;

#[cfg(test)]
mod authorization_tests;

#[cfg(test)]
mod aggregation_tests;

#[cfg(test)]
mod inspection_tests;

#[cfg(test)]
mod policy_migration_tests;

#[cfg(test)]
mod validation_tests;

pub use model::{
    AdminUpdatePolicySnapshot, AssetInspectionStatus, CreateSourceInput, DownloadAggregationReport,
    DownloadHistoryItem, InstalledIdentityEntryInput, LatestUpdate, PublicAsset, PublicRelease,
    PublicSource, PublishUpdatePolicyInput, PublishedAssetInspectionReport, PublishedUpdatePolicy,
    RecordInstalledIdentitiesInput, RecordInstalledIdentitiesResult, ReleaseSource,
    SaveUpdatePolicyDraftInput, SourceKind, UpdateAsset, UpdateCheckQuery, UpdateCheckResponse,
    UpdateIdentityStatus, UpdatePolicyDraft, UpdatePolicyTargetRelease, UpdateSource,
    UpdateSourceInput,
};
pub use router::{account_router, management_router, public_router, update_router};
pub use service::Service;
pub use upload_file::PreparedLocalUpload;
