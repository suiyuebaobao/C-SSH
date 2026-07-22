//! 管理安装资产的多来源元数据与公开下载分发。
//! 本包不依赖其它业务包，版本与资产仅通过只读 SQL 投影关联。

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
mod validation_tests;

pub use model::{
    AssetInspectionStatus, CreateSourceInput, DownloadAggregationReport, DownloadHistoryItem,
    PublicAsset, PublicRelease, PublicSource, PublishedAssetInspectionReport, ReleaseSource,
    SourceKind, UpdateSourceInput,
};
pub use router::{account_router, management_router, public_router};
pub use service::Service;
