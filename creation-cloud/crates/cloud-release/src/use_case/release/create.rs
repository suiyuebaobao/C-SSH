//! 校验创建版本输入并委托独立 repository 持久化。

use cloud_domain::{AdminActor, AppResult};

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
    Ok(CreateReleaseInput {
        version: validation::required_text(&input.version, "版本号", 64)?,
        channel: input.channel,
        title_zh: validation::required_text(&input.title_zh, "中文标题", 200)?,
        title_en: validation::required_text(&input.title_en, "英文标题", 200)?,
        notes_zh: validation::required_text(&input.notes_zh, "中文发布说明", 20_000)?,
        notes_en: validation::required_text(&input.notes_en, "英文发布说明", 20_000)?,
    })
}
