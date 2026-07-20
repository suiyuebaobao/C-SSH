//! 校验来源类型与位置，并对本站文件做真实身份核验。

use cloud_domain::{AdminActor, AppError, AppResult};

use crate::{
    CreateSourceInput, ReleaseSource, Service, SourceKind, authorization, repository, validation,
};

impl Service {
    pub async fn create_source(
        &self,
        actor: &AdminActor,
        input: CreateSourceInput,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        if input.source_kind == SourceKind::Local {
            return Err(AppError::Validation(
                "本站来源必须通过资产上传接口创建".into(),
            ));
        }
        let input = normalize(input)?;
        let asset_id = input.asset_id;
        let asset = repository::asset::get(&self.pool, asset_id).await?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let release_status =
            repository::release_lock::execute(&mut transaction, asset.release_id).await?;
        let locked_asset = repository::asset_lock::execute(&mut transaction, asset_id).await?;
        if locked_asset.release_id != asset.release_id {
            return Err(AppError::Conflict("资产所属版本已经变化".into()));
        }
        if matches!(release_status.as_str(), "revoked" | "hidden") {
            return Err(AppError::Conflict("已撤销或隐藏版本不能新增来源".into()));
        }
        let source =
            repository::source::create::execute_in_transaction(&mut transaction, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(source)
    }
}

pub(crate) fn normalize(input: CreateSourceInput) -> AppResult<CreateSourceInput> {
    let provider_name = validation::required_text(&input.provider_name, "来源名称", 100)?;
    let sort_order = validation::sort_order(input.sort_order)?;
    let (local_path, external_url) = match input.source_kind {
        SourceKind::Local => {
            if input.external_url.is_some() {
                return Err(AppError::Validation("本站来源不能同时包含外部 URL".into()));
            }
            let path = input
                .local_path
                .as_deref()
                .ok_or_else(|| AppError::Validation("本站来源缺少相对路径".into()))?;
            (Some(validation::local_path(path)?), None)
        }
        SourceKind::External => {
            if input.local_path.is_some() {
                return Err(AppError::Validation("外部来源不能同时包含本站路径".into()));
            }
            let url = input
                .external_url
                .as_deref()
                .ok_or_else(|| AppError::Validation("外部来源缺少 HTTPS URL".into()))?;
            (None, Some(validation::external_url(url)?))
        }
    };

    Ok(CreateSourceInput {
        asset_id: validation::valid_id(input.asset_id, "资产标识")?,
        source_kind: input.source_kind,
        provider_name,
        local_path,
        external_url,
        sort_order,
        enabled: input.enabled,
    })
}
