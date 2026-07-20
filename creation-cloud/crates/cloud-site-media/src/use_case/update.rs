//! 只允许修改草稿的双语替代文本，文件身份和发布历史保持不可变。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{Service, SiteMedia, UpdateSiteMediaInput, repository, validation};

impl Service {
    pub async fn update(
        &self,
        _actor: &AdminActor,
        media_id: Uuid,
        input: UpdateSiteMediaInput,
    ) -> AppResult<SiteMedia> {
        let input = normalize_input(input)?;
        repository::update::execute(self.pool(), validation::valid_id(media_id)?, &input).await
    }
}

pub(super) fn normalize_input(input: UpdateSiteMediaInput) -> AppResult<UpdateSiteMediaInput> {
    if input.alt_zh.is_none() && input.alt_en.is_none() {
        return Err(AppError::Validation("至少提供一个替代文本字段".into()));
    }
    Ok(UpdateSiteMediaInput {
        alt_zh: input
            .alt_zh
            .map(|value| validation::alt_text(&value, "中文替代文本"))
            .transpose()?,
        alt_en: input
            .alt_en
            .map(|value| validation::alt_text(&value, "英文替代文本"))
            .transpose()?,
    })
}
