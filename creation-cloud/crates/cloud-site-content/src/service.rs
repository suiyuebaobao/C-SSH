use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use cloud_site::Locale;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    CreateSiteContentInput, PublicSiteContent, SiteContentDocumentKey, SiteContentListQuery,
    SiteContentPayload, SiteContentRevision, SiteContentTransitionInput, UpdateSiteContentInput,
    repository, validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    site_media: cloud_site_media::Service,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool, site_media: cloud_site_media::Service) -> Self {
        Self { pool, site_media }
    }

    pub async fn ready(&self) -> AppResult<()> {
        sqlx::query_scalar::<_, i32>("SELECT 1 FROM site_content_revisions LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Storage("站点内容存储不可用".into()))?;
        Ok(())
    }

    pub async fn list(
        &self,
        actor: &AdminActor,
        query: SiteContentListQuery,
    ) -> AppResult<Vec<SiteContentRevision>> {
        require_actor(actor)?;
        repository::list(&self.pool, query.document_key, query.locale).await
    }

    pub async fn get(&self, actor: &AdminActor, id: Uuid) -> AppResult<SiteContentRevision> {
        require_actor(actor)?;
        repository::get(&self.pool, validation::id(id)?).await
    }

    pub async fn published(
        &self,
        key: SiteContentDocumentKey,
        locale: Locale,
    ) -> AppResult<Option<SiteContentRevision>> {
        repository::published(&self.pool, key, locale).await
    }

    pub async fn public_content(
        &self,
        key: SiteContentDocumentKey,
        locale: Locale,
    ) -> AppResult<PublicSiteContent> {
        if let Some(record) = repository::published(&self.pool, key, locale).await? {
            return Ok(PublicSiteContent::Published(record.content));
        }
        if repository::has_publication_history(&self.pool, key, locale).await? {
            Ok(PublicSiteContent::Unavailable)
        } else {
            Ok(PublicSiteContent::LegacyFallback)
        }
    }

    pub async fn create_draft(
        &self,
        actor: &AdminActor,
        input: CreateSiteContentInput,
    ) -> AppResult<SiteContentRevision> {
        let actor_id = require_actor(actor)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        repository::lock_scope(&mut transaction, input.document_key, input.locale).await?;
        let content = match input.content {
            Some(content) => content,
            None => repository::published_in_transaction(
                &mut transaction,
                input.document_key,
                input.locale,
            )
            .await?
            .map_or_else(
                || SiteContentPayload::compiled(input.document_key, input.locale),
                |record| record.content,
            ),
        };
        let content = validation::payload(input.document_key, input.locale, content)?;
        let record = repository::create_draft(
            &mut transaction,
            actor_id,
            input.document_key,
            input.locale,
            &content,
        )
        .await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.draft_created",
            &record,
            validation::field_count(&content),
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn update_draft(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateSiteContentInput,
    ) -> AppResult<SiteContentRevision> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::editable_draft(&current, input.expected_revision)?;
        let content = validation::payload(current.document_key, current.locale, input.content)?;
        let record =
            repository::update_draft(&mut transaction, id, input.expected_revision, &content)
                .await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.draft_updated",
            &record,
            validation::field_count(&content),
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn publish(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: SiteContentTransitionInput,
    ) -> AppResult<SiteContentRevision> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let identity = repository::get(&self.pool, id).await?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        repository::lock_scope(&mut transaction, identity.document_key, identity.locale).await?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::publishable(&current, input.expected_revision)?;
        validation::payload(
            current.document_key,
            current.locale,
            current.content.clone(),
        )?;
        self.validate_media(&current.content).await?;
        repository::revoke_current(
            &mut transaction,
            current.document_key,
            current.locale,
            Some(id),
        )
        .await?;
        let record = repository::publish(&mut transaction, id, input.expected_revision).await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.published",
            &record,
            validation::field_count(&record.content),
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn revoke(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: SiteContentTransitionInput,
    ) -> AppResult<SiteContentRevision> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let identity = repository::get(&self.pool, id).await?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        repository::lock_scope(&mut transaction, identity.document_key, identity.locale).await?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::revocable(&current, input.expected_revision)?;
        let record = repository::revoke(&mut transaction, id, input.expected_revision).await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.revoked",
            &record,
            validation::field_count(&record.content),
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    pub async fn delete_draft(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: SiteContentTransitionInput,
    ) -> AppResult<()> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        let current = repository::lock(&mut transaction, id).await?;
        validation::editable_draft(&current, input.expected_revision)?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.draft_deleted",
            &current,
            validation::field_count(&current.content),
        )
        .await?;
        repository::delete_draft(&mut transaction, id, input.expected_revision).await?;
        commit(transaction).await?;
        Ok(())
    }

    pub async fn rollback(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: SiteContentTransitionInput,
    ) -> AppResult<SiteContentRevision> {
        let actor_id = require_actor(actor)?;
        let id = validation::id(id)?;
        let identity = repository::get(&self.pool, id).await?;
        let mut transaction = self.pool.begin().await.map_err(transaction_error)?;
        repository::lock_scope(&mut transaction, identity.document_key, identity.locale).await?;
        let source = repository::lock(&mut transaction, id).await?;
        validation::rollback_source(&source, input.expected_revision)?;
        validation::payload(source.document_key, source.locale, source.content.clone())?;
        self.validate_media(&source.content).await?;
        repository::revoke_current(&mut transaction, source.document_key, source.locale, None)
            .await?;
        let record = repository::rollback(&mut transaction, actor_id, &source).await?;
        repository::audit(
            &mut transaction,
            actor_id,
            "site_content.rolled_back",
            &record,
            validation::field_count(&record.content),
        )
        .await?;
        commit(transaction).await?;
        Ok(record)
    }

    async fn validate_media(&self, content: &SiteContentPayload) -> AppResult<()> {
        if content.media_slot().is_none() {
            return Ok(());
        }
        match self.site_media.current_home_qr().await {
            Ok(_) => Ok(()),
            Err(AppError::NotFound(_)) => Err(AppError::Validation(
                "引用 home_qr 前必须先发布受控首页二维码".into(),
            )),
            Err(error) => Err(error),
        }
    }
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let id = actor.account_id();
    if id.is_nil() {
        return Err(AppError::Unauthorized("管理员身份无效".into()));
    }
    Ok(id)
}

async fn commit(transaction: cloud_store::Transaction<'_, cloud_store::Postgres>) -> AppResult<()> {
    transaction.commit().await.map_err(transaction_error)?;
    mark_semantic_audit_recorded();
    Ok(())
}

fn transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("站点内容事务失败".into())
}
