//! 汇总版本与资产的公开值对象及请求输入。

mod asset;
mod release;

pub(crate) use asset::AssetRow;
pub use asset::{CreateAssetInput, ReleaseAsset, UpdateAssetInput};
pub(crate) use release::ReleaseRow;
pub use release::{
    CreateReleaseInput, CreateRevisionInput, Release, ReleaseChannel, ReleaseStatus,
    UpdateReleaseInput,
};
