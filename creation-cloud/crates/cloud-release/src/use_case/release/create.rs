//! 校验创建版本输入并委托独立 repository 持久化。

use cloud_domain::{
    AdminActor, AppError, AppResult, MAX_SEMANTIC_VERSION_LENGTH, normalize_semantic_version,
};

use crate::{CreateReleaseInput, Release, Service, authorization, repository, validation};

impl Service {
    pub async fn create_release(
        &self,
        actor: &AdminActor,
        input: CreateReleaseInput,
    ) -> AppResult<Release> {
        authorization::require(actor)?;
        repository::release::create::execute(&self.pool, &normalize(input)?).await
    }
}

pub(crate) fn normalize(input: CreateReleaseInput) -> AppResult<CreateReleaseInput> {
    let version =
        validation::required_text(&input.version, "版本号", MAX_SEMANTIC_VERSION_LENGTH + 1)?;
    let (version, _) = normalize_semantic_version(&version)
        .ok_or_else(|| AppError::Validation("版本号必须是有效的 SemVer 2.0.0 语义版本".into()))?;
    Ok(CreateReleaseInput {
        version,
        channel: input.channel,
        title_zh: validation::required_text(&input.title_zh, "中文标题", 200)?,
        title_en: validation::required_text(&input.title_en, "英文标题", 200)?,
        notes_zh: validation::required_text(&input.notes_zh, "中文发布说明", 20_000)?,
        notes_en: validation::required_text(&input.notes_en, "英文发布说明", 20_000)?,
    })
}
