//! 校验资产身份并确保所属版本尚未发布。

use cloud_domain::{AdminActor, AppError, AppResult};

use crate::{CreateAssetInput, ReleaseAsset, Service, authorization, repository, validation};

impl Service {
    pub async fn create_asset(
        &self,
        actor: &AdminActor,
        input: CreateAssetInput,
    ) -> AppResult<ReleaseAsset> {
        authorization::require(actor)?;
        let input = normalize(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release =
            repository::release::lock::execute(&mut transaction, input.release_id).await?;
        if !release.status.allows_asset_mutation() {
            return Err(AppError::Conflict("已发布版本不能新增资产".into()));
        }
        let asset = repository::asset::create::execute(&mut transaction, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(asset)
    }
}

pub(crate) fn normalize(input: CreateAssetInput) -> AppResult<CreateAssetInput> {
    let normalized = CreateAssetInput {
        release_id: validation::valid_id(input.release_id, "版本标识")?,
        platform: validation::platform(&input.platform)?,
        architecture: validation::architecture(&input.architecture)?,
        package_kind: validation::package_kind(&input.package_kind)?,
        file_name: validation::file_name(&input.file_name)?,
        byte_size: validation::byte_size(input.byte_size)?,
        sha256: validation::sha256(&input.sha256)?,
    };
    validation::asset_identity(&normalized.platform, &normalized.package_kind)?;
    Ok(normalized)
}
