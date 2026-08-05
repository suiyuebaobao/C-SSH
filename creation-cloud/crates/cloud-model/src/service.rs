//! 全局模型目录用例。

use cloud_domain::{AdminActor, AppError, AppResult, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    CreateGlobalModelInput, GlobalModel, PublicGlobalModel, ReplaceGlobalModelInput, repository,
    validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_public(&self, page: PageQuery) -> AppResult<Page<PublicGlobalModel>> {
        let page = repository::list_public(&self.pool, page).await?;
        Ok(Page {
            items: page
                .items
                .into_iter()
                .map(PublicGlobalModel::from)
                .collect(),
            page: page.page,
            size: page.size,
            total: page.total,
        })
    }

    pub async fn get_public(&self, id: Uuid) -> AppResult<PublicGlobalModel> {
        repository::get_public(&self.pool, validation::model_id(id)?)
            .await
            .map(PublicGlobalModel::from)
    }

    /// 补齐系统预置模型，并修订从未被管理员编辑的旧预置。
    ///
    /// 该操作只更新 `system_seeded=true + revision=1` 的活动记录；
    /// 不覆盖管理员编辑/停用，不复活删除项。没有有效管理员时保持原状。
    pub async fn seed_system_catalog(&self) -> AppResult<u64> {
        repository::seed_system_catalog(&self.pool).await
    }

    pub async fn list_admin(
        &self,
        actor: &AdminActor,
        page: PageQuery,
    ) -> AppResult<Page<GlobalModel>> {
        require_actor(actor)?;
        repository::list_admin(&self.pool, page).await
    }

    pub async fn get_admin(&self, actor: &AdminActor, id: Uuid) -> AppResult<GlobalModel> {
        require_actor(actor)?;
        repository::get_admin(&self.pool, validation::model_id(id)?).await
    }

    pub async fn create_admin(
        &self,
        actor: &AdminActor,
        input: CreateGlobalModelInput,
    ) -> AppResult<GlobalModel> {
        let actor_id = require_actor(actor)?;
        repository::create(&self.pool, actor_id, validation::create(input)?).await
    }

    pub async fn replace_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: ReplaceGlobalModelInput,
    ) -> AppResult<GlobalModel> {
        let actor_id = require_actor(actor)?;
        let id = validation::model_id(id)?;
        let (expected_revision, value) = validation::replace(input)?;
        repository::replace(&self.pool, actor_id, id, expected_revision, value).await
    }

    pub async fn delete_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        expected_revision: i64,
    ) -> AppResult<()> {
        let actor_id = require_actor(actor)?;
        repository::delete(
            &self.pool,
            actor_id,
            validation::model_id(id)?,
            validation::revision(expected_revision)?,
        )
        .await
    }
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let id = actor.account_id();
    if id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".to_owned()))
    } else {
        Ok(id)
    }
}
