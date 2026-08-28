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

#[doc(hidden)]
pub async fn create_asset_in_transaction(
    actor: &cloud_domain::AdminActor,
    transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
    input: CreateAssetInput,
) -> cloud_domain::AppResult<ReleaseAsset> {
    use_case::asset::create::create_in_transaction(actor, transaction, input).await
}

#[doc(hidden)]
pub async fn record_installed_sha256_in_transaction(
    actor: &cloud_domain::AdminActor,
    transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
    asset_id: uuid::Uuid,
    installed_sha256: &str,
) -> cloud_domain::AppResult<ReleaseAsset> {
    use_case::asset::installed_identity::record_in_transaction(
        actor,
        transaction,
        asset_id,
        installed_sha256,
    )
    .await
}
