//! 全局模型目录与个人客户端密文用例。

use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    CreateGlobalModelInput, GlobalModel, ModelSecret, ReplaceGlobalModelInput, repository,
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

    pub async fn list_public(&self, page: PageQuery) -> AppResult<Page<GlobalModel>> {
        repository::list_public(&self.pool, page).await
    }

    pub async fn get_public(&self, id: Uuid) -> AppResult<GlobalModel> {
        repository::get_public(&self.pool, validation::model_id(id)?).await
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

    pub async fn get_secret(
        &self,
        session: &AuthenticatedSession,
        model_id: Uuid,
    ) -> AppResult<ModelSecret> {
        repository::get_secret(
            &self.pool,
            require_account(session)?,
            validation::model_id(model_id)?,
        )
        .await
    }

    pub async fn put_secret(
        &self,
        session: &AuthenticatedSession,
        model_id: Uuid,
        ciphertext: &str,
        expected_revision: Option<i64>,
    ) -> AppResult<ModelSecret> {
        let (account_id, device_id) = require_device(session)?;
        let expected_revision = expected_revision.map(validation::revision).transpose()?;
        repository::put_secret(
            &self.pool,
            account_id,
            device_id,
            validation::model_id(model_id)?,
            validation::ciphertext(ciphertext)?,
            expected_revision,
        )
        .await
    }

    pub async fn delete_secret(
        &self,
        session: &AuthenticatedSession,
        model_id: Uuid,
        expected_revision: i64,
    ) -> AppResult<ModelSecret> {
        let (account_id, device_id) = require_device(session)?;
        repository::delete_secret(
            &self.pool,
            account_id,
            device_id,
            validation::model_id(model_id)?,
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

fn require_account(session: &AuthenticatedSession) -> AppResult<Uuid> {
    if session.account_id.is_nil() {
        Err(AppError::Unauthorized("账号身份无效".to_owned()))
    } else {
        Ok(session.account_id)
    }
}

fn require_device(session: &AuthenticatedSession) -> AppResult<(Uuid, Uuid)> {
    let account_id = require_account(session)?;
    let device_id = session
        .device_id
        .filter(|id| !id.is_nil())
        .ok_or_else(|| AppError::Forbidden("需要绑定有效设备".to_owned()))?;
    Ok((account_id, device_id))
}
