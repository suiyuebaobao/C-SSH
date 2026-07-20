//! 校验并重编码管理员上传，在数据库事务提交前原子落位受控 PNG 文件。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    CreateSiteMediaInput, Service, SiteMedia, finalization, image_validation,
    model::NewSiteMedia,
    repository,
    storage::{self, StagedFile},
    validation,
};

pub(super) struct PreparedUpload {
    pub(super) image: image_validation::ValidatedImage,
    pub(super) alt_zh: String,
    pub(super) alt_en: String,
}

impl Service {
    pub async fn create(
        &self,
        actor: &AdminActor,
        input: CreateSiteMediaInput,
    ) -> AppResult<SiteMedia> {
        let prepared = prepare(input)?;
        let byte_size = i64::try_from(prepared.image.png.len())
            .map_err(|_| AppError::Internal("站点媒体长度无法表示".into()))?;
        let width = i32::try_from(prepared.image.width)
            .map_err(|_| AppError::Internal("站点媒体宽度无法表示".into()))?;
        let height = i32::try_from(prepared.image.height)
            .map_err(|_| AppError::Internal("站点媒体高度无法表示".into()))?;

        let staged = storage::stage(self.site_media_root(), &prepared.image.png).await?;
        let new_media = NewSiteMedia {
            id: Uuid::now_v7(),
            storage_key: staged.storage_key.clone(),
            byte_size,
            sha256: prepared.image.sha256,
            width,
            height,
            alt_zh: prepared.alt_zh,
            alt_en: prepared.alt_en,
            created_by: actor.account_id(),
        };
        create_transaction(self, staged, &new_media).await
    }
}

pub(super) fn prepare(input: CreateSiteMediaInput) -> AppResult<PreparedUpload> {
    Ok(PreparedUpload {
        alt_zh: validation::alt_text(&input.alt_zh, "中文替代文本")?,
        alt_en: validation::alt_text(&input.alt_en, "英文替代文本")?,
        image: image_validation::validate_and_reencode(&input.declared_content_type, &input.bytes)?,
    })
}

async fn create_transaction(
    service: &Service,
    mut staged: StagedFile,
    input: &NewSiteMedia,
) -> AppResult<SiteMedia> {
    let mut transaction = match service.pool().begin().await {
        Ok(transaction) => transaction,
        Err(_) => {
            staged.cleanup().await;
            return Err(AppError::Storage("无法开始站点媒体创建事务".into()));
        }
    };
    let media = match repository::create::execute(&mut transaction, input).await {
        Ok(media) => media,
        Err(error) => {
            let _ = transaction.rollback().await;
            staged.cleanup().await;
            return Err(error);
        }
    };
    if let Err(error) = staged.commit().await {
        let _ = transaction.rollback().await;
        staged.cleanup().await;
        return Err(error);
    }
    finalization::create(service.pool().clone(), transaction, staged, media).await
}
