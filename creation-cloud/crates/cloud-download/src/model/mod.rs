//! 汇总来源、公开清单和内部下载目标模型。

mod aggregation;
mod history;
mod inspection;
mod public;
mod source;
mod target;

pub use aggregation::DownloadAggregationReport;
pub(crate) use aggregation::{DownloadAggregateBucket, DownloadAudience, PendingDownloadEvent};
pub use history::DownloadHistoryItem;
pub(crate) use inspection::{AssetInspectionObservation, FileInspection, PublishedLocalAsset};
pub use inspection::{AssetInspectionStatus, PublishedAssetInspectionReport};
pub(crate) use public::PublicCatalogRow;
pub use public::{PublicAsset, PublicRelease, PublicSource};
pub(crate) use source::SourceRow;
pub use source::{CreateSourceInput, ReleaseSource, SourceKind, UpdateSourceInput};
pub(crate) use target::{AssetRecord, DownloadTarget, LockedAssetRecord};
