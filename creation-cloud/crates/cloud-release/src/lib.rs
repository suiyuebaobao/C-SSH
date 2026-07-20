//! 管理 Creation Cloud 的版本与不可变安装资产身份。
//! 下载来源和文件传输由独立的 `cloud-download` 包负责。

mod authorization;
mod handler;
mod model;
mod repository;
mod router;
mod service;
mod use_case;
mod validation;

#[cfg(test)]
mod migration_tests;

#[cfg(test)]
mod router_tests;

#[cfg(test)]
mod authorization_tests;

pub use model::{
    CreateAssetInput, CreateReleaseInput, Release, ReleaseAsset, ReleaseChannel, ReleaseStatus,
    UpdateAssetInput, UpdateReleaseInput,
};
pub use router::router;
pub use service::Service;
