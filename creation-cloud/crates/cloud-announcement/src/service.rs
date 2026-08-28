use cloud_domain::{
    AdminActor, AppError, AppResult, Page, PageQuery, mark_semantic_audit_recorded,
};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    Announcement, AnnouncementLocale, AnnouncementStatus, CreateAnnouncementInput,
    CurrentAnnouncementResponse, ReplaceAnnouncementInput, TransitionAnnouncementInput, repository,
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

    pub async fn current(
        &self,
        locale: AnnouncementLocale,
    ) -> AppResult<CurrentAnnouncementResponse> {
        let current = repository::current(&self.pool).await?;
        let announcement = current
            .announcement
            .map(|record| record.localized(locale))
            .transpose()?;
        Ok(CurrentAnnouncementResponse {
            revision: current.public_revision,
            announcement,
        })
    }

    pub async fn list_admin(
        &self,
        actor: &AdminActor,
        page: PageQuery,
    ) -> AppResult<Page<Announcement>> {
        require_actor(actor)?;
        repository::list(&self.pool, page).await
    }

    pub async fn get_admin(&self, actor: &AdminActor, id: Uuid) -> AppResult<Announcement> {
        require_actor(actor)?;
        repository::get(&self.pool, validation::id(id)?).await
    }

    pub async fn create_admin(
        &self,
        actor: &AdminActor,
        input: CreateAnnouncementInput,
    ) -> AppResult<Announcement> {
        let actor_id = require_actor(actor)?;
        let value = validation::create(input)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let record = repository::create(&mut transaction, actor_id, &value).await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "announcement.created",
            record.id,
            record.status.as_str(),
            record.priority,
            record.revision,
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn replace_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: ReplaceAnnouncementInput,
    ) -> AppResult<Announcement> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let (expected_revision, value) = validation::replace(input)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let publication = repository::lock_publication(&mut transaction).await?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::editable(&current, expected_revision)?;
        let record = match current.status {
            AnnouncementStatus::Draft => {
                repository::replace_draft(&mut transaction, actor_id, id, expected_revision, &value)
                    .await?
            }
            AnnouncementStatus::Published => {
                if publication.current_announcement_id != Some(id) {
                    return Err(AppError::Conflict(
                        "公告已不是当前公开公告，请刷新后重试".to_owned(),
                    ));
                }
                repository::hide(&mut transaction, actor_id, id, expected_revision).await?;
                let draft = repository::create(&mut transaction, actor_id, &value).await?;
                let replacement =
                    repository::publish(&mut transaction, actor_id, draft.id, draft.revision)
                        .await?;
                repository::advance_publication(
                    &mut transaction,
                    publication.public_revision,
                    Some(replacement.id),
                )
                .await?;
                replacement
            }
            AnnouncementStatus::Hidden => {
                return Err(AppError::Conflict("已隐藏公告不能编辑".to_owned()));
            }
        };
        repository::audit(
            &mut transaction,
            actor_id,
            "announcement.updated",
            record.id,
            record.status.as_str(),
            record.priority,
            record.revision,
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn delete_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: TransitionAnnouncementInput,
    ) -> AppResult<()> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let expected_revision = validation::revision(input.expected_revision)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let publication = repository::lock_publication(&mut transaction).await?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::deletable(&current, expected_revision)?;
        let audit_revision = match current.status {
            AnnouncementStatus::Draft => {
                repository::delete_draft(&mut transaction, id, expected_revision).await?;
                current.revision
            }
            AnnouncementStatus::Published => {
                if publication.current_announcement_id != Some(id) {
                    return Err(AppError::Conflict(
                        "公告已不是当前公开公告，请刷新后重试".to_owned(),
                    ));
                }
                let hidden =
                    repository::hide(&mut transaction, actor_id, id, expected_revision).await?;
                repository::advance_publication(
                    &mut transaction,
                    publication.public_revision,
                    None,
                )
                .await?;
                hidden.revision
            }
            AnnouncementStatus::Hidden => {
                return Err(AppError::Conflict("已隐藏公告不能删除".to_owned()));
            }
        };
        repository::audit(
            &mut transaction,
            actor_id,
            "announcement.deleted",
            current.id,
            "deleted",
            current.priority,
            audit_revision,
        )
        .await?;
        commit(transaction).await
    }

    pub async fn publish_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: TransitionAnnouncementInput,
    ) -> AppResult<Announcement> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let expected_revision = validation::revision(input.expected_revision)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let publication = repository::lock_publication(&mut transaction).await?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::publishable(&current, expected_revision)?;
        if let Some(current_id) = publication.current_announcement_id {
            repository::hide_current(&mut transaction, actor_id, current_id).await?;
        }
        let record = repository::publish(&mut transaction, actor_id, id, expected_revision).await?;
        repository::advance_publication(
            &mut transaction,
            publication.public_revision,
            Some(record.id),
        )
        .await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "announcement.published",
            record.id,
            record.status.as_str(),
            record.priority,
            record.revision,
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn hide_admin(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: TransitionAnnouncementInput,
    ) -> AppResult<Announcement> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let expected_revision = validation::revision(input.expected_revision)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let publication = repository::lock_publication(&mut transaction).await?;
        if publication.current_announcement_id != Some(id) {
            return Err(AppError::Conflict(
                "公告已不是当前公开公告，请刷新后重试".to_owned(),
            ));
        }
        let current = repository::lock(&mut transaction, id).await?;
        validation::hideable(&current, expected_revision)?;
        let record = repository::hide(&mut transaction, actor_id, id, expected_revision).await?;
        repository::advance_publication(&mut transaction, publication.public_revision, None)
            .await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "announcement.hidden",
            record.id,
            record.status.as_str(),
            record.priority,
            record.revision,
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let actor_id = actor.account_id();
    if actor_id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".to_owned()))
    } else {
        Ok(actor_id)
    }
}

async fn commit(transaction: cloud_store::Transaction<'_, cloud_store::Postgres>) -> AppResult<()> {
    transaction.commit().await.map_err(transaction_error)?;
    mark_semantic_audit_recorded();
    Ok(())
}

fn transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("公告事务失败".to_owned())
}
