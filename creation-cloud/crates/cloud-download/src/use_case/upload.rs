//! 编排本站资产上传、原子落盘与来源记录提交。

use axum::extract::Multipart;
use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    CreateSourceInput, ReleaseSource, Service, SourceKind, authorization, repository, upload_file,
    validation,
};

impl Service {
    pub async fn upload_asset(
        &self,
        actor: &AdminActor,
        asset_id: Uuid,
        mut multipart: Multipart,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        let asset_id = validation::valid_id(asset_id, "资产标识")?;
        let asset = repository::asset::get(&self.pool, asset_id).await?;
        if matches!(asset.release_status.as_str(), "revoked" | "hidden") {
            return Err(AppError::Conflict("已撤销或隐藏版本不能上传资产".into()));
        }
        let expected_size = u64::try_from(asset.byte_size)
            .map_err(|_| AppError::Conflict("资产身份大小无效".into()))?;
        upload_file::validate_asset_identity(expected_size, &asset.sha256)?;

        let layout = upload_file::UploadLayout::prepare(self.download_root.as_path()).await?;
        let object_id = Uuid::now_v7();
        let temp_path = layout.temp_path(object_id);
        let mut cleanup = upload_file::CleanupFile::new(temp_path.clone());
        let received =
            upload_file::receive(&mut multipart, &temp_path, expected_size, &asset.sha256).await?;
        let (_final_path, relative_path) =
            layout.promote(&temp_path, object_id, &mut cleanup).await?;

        let input = CreateSourceInput {
            asset_id,
            source_kind: SourceKind::Local,
            provider_name: received.provider_name,
            local_path: Some(validation::local_path(&relative_path)?),
            external_url: None,
            sort_order: 0,
            enabled: true,
        };
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release_status =
            repository::release_lock::execute(&mut transaction, asset.release_id).await?;
        let locked = repository::asset_lock::execute(&mut transaction, asset_id).await?;
        if locked.release_id != asset.release_id
            || locked.byte_size != asset.byte_size
            || !locked.sha256.eq_ignore_ascii_case(&asset.sha256)
        {
            return Err(AppError::Conflict("上传期间资产身份已经变化".into()));
        }
        if matches!(release_status.as_str(), "revoked" | "hidden") {
            return Err(AppError::Conflict("已撤销或隐藏版本不能上传资产".into()));
        }
        let source =
            repository::source::create::execute_in_transaction(&mut transaction, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        cleanup.disarm();
        Ok(source)
    }
}
