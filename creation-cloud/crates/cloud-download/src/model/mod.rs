//! 汇总来源、公开清单和内部下载目标模型。

mod public;
mod source;
mod target;

pub(crate) use public::PublicCatalogRow;
pub use public::{PublicAsset, PublicRelease, PublicSource};
pub(crate) use source::SourceRow;
pub use source::{CreateSourceInput, ReleaseSource, SourceKind, UpdateSourceInput};
pub(crate) use target::{AssetRecord, DownloadTarget, LockedAssetRecord};
