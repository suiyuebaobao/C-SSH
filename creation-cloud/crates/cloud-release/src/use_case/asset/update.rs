//! 更新草稿或校验中版本的资产身份。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{ReleaseAsset, Service, UpdateAssetInput, authorization, repository, validation};

impl Service {
    pub async fn update_asset(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateAssetInput,
    ) -> AppResult<ReleaseAsset> {
        authorization::require(actor)?;
        let id = validation::valid_id(id, "资产标识")?;
        let locator = repository::asset::get::execute(&self.pool, id).await?;
        let input = normalize(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release =
            repository::release::lock::execute(&mut transaction, locator.release_id).await?;
        let current = repository::asset::lock::execute(&mut transaction, id).await?;
        if current.release_id != locator.release_id {
            return Err(AppError::Conflict("资产所属版本已经变化".into()));
        }
        if !release.status.allows_asset_mutation() {
            return Err(AppError::Conflict("已发布版本的资产不可原地覆盖".into()));
        }
        let platform = input.platform.as_deref().unwrap_or(&current.platform);
        let package_kind = input
            .package_kind
            .as_deref()
            .unwrap_or(&current.package_kind);
        validation::asset_identity(platform, package_kind)?;
        let asset = repository::asset::update::execute(&mut transaction, id, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(asset)
    }
}

pub(crate) fn normalize(input: UpdateAssetInput) -> AppResult<UpdateAssetInput> {
    let normalized = UpdateAssetInput {
        platform: input
            .platform
            .as_deref()
            .map(validation::platform)
            .transpose()?,
        architecture: input
            .architecture
            .as_deref()
            .map(validation::architecture)
            .transpose()?,
        package_kind: input
            .package_kind
            .as_deref()
            .map(validation::package_kind)
            .transpose()?,
        file_name: input
            .file_name
            .as_deref()
            .map(validation::file_name)
            .transpose()?,
        byte_size: input.byte_size.map(validation::byte_size).transpose()?,
        sha256: input
            .sha256
            .as_deref()
            .map(validation::sha256)
            .transpose()?,
    };
    if normalized.platform.is_none()
        && normalized.architecture.is_none()
        && normalized.package_kind.is_none()
        && normalized.file_name.is_none()
        && normalized.byte_size.is_none()
        && normalized.sha256.is_none()
    {
        return Err(AppError::Validation("资产更新内容不能为空".into()));
    }
    Ok(normalized)
}
