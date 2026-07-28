use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

use crate::{
    CreateSeoTopicInput, SeoLocale, SeoTopic, Service, UpdateSeoTopicInput, authorization,
    repository, validation,
};

const ENABLED_LIMIT: i64 = 12;

impl Service {
    pub async fn public_topics(&self, locale: SeoLocale) -> AppResult<Vec<SeoTopic>> {
        repository::public_list(&self.pool, locale).await
    }

    pub async fn list_topics(&self, actor: &AdminActor) -> AppResult<Vec<SeoTopic>> {
        authorization::require(actor)?;
        repository::list(&self.pool).await
    }

    pub async fn create_topic(
        &self,
        actor: &AdminActor,
        input: CreateSeoTopicInput,
    ) -> AppResult<SeoTopic> {
        let created_by = authorization::require(actor)?;
        let input = normalize_create(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::transaction_error)?;
        repository::lock_writes(&mut transaction).await?;
        enforce_enabled_limit(&mut transaction, input.locale, input.enabled, None).await?;
        let topic = repository::create(&mut transaction, created_by, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::transaction_error)?;
        Ok(topic)
    }

    pub async fn update_topic(
        &self,
        actor: &AdminActor,
        id: Uuid,
        input: UpdateSeoTopicInput,
    ) -> AppResult<SeoTopic> {
        authorization::require(actor)?;
        let id = validation::id(id)?;
        let input = normalize_update(input)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::transaction_error)?;
        repository::lock_writes(&mut transaction).await?;
        let current = repository::lock_one(&mut transaction, id).await?;
        let final_locale = input.locale.unwrap_or(current.locale);
        let final_enabled = input.enabled.unwrap_or(current.enabled);
        enforce_enabled_limit(&mut transaction, final_locale, final_enabled, Some(id)).await?;
        let topic = repository::update(&mut transaction, id, &input).await?;
        transaction
            .commit()
            .await
            .map_err(repository::transaction_error)?;
        Ok(topic)
    }

    pub async fn delete_topic(&self, actor: &AdminActor, id: Uuid) -> AppResult<()> {
        authorization::require(actor)?;
        let id = validation::id(id)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::transaction_error)?;
        repository::lock_writes(&mut transaction).await?;
        repository::delete(&mut transaction, id).await?;
        transaction
            .commit()
            .await
            .map_err(repository::transaction_error)
    }
}

fn normalize_create(mut input: CreateSeoTopicInput) -> AppResult<CreateSeoTopicInput> {
    input.phrase = validation::phrase(&input.phrase)?;
    Ok(input)
}

pub(crate) fn normalize_update(mut input: UpdateSeoTopicInput) -> AppResult<UpdateSeoTopicInput> {
    if input.locale.is_none()
        && input.phrase.is_none()
        && input.sort_order.is_none()
        && input.enabled.is_none()
    {
        return Err(AppError::Validation(
            "SEO 主题词更新内容不能为空".to_owned(),
        ));
    }
    input.phrase = input
        .phrase
        .map(|phrase| validation::phrase(&phrase))
        .transpose()?;
    Ok(input)
}

async fn enforce_enabled_limit(
    transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
    locale: SeoLocale,
    enabled: bool,
    excluding: Option<Uuid>,
) -> AppResult<()> {
    let enabled_count = repository::enabled_count(transaction, locale, excluding).await?;
    ensure_enabled_capacity(enabled, enabled_count)
}

pub(crate) fn ensure_enabled_capacity(enabled: bool, enabled_count: i64) -> AppResult<()> {
    if !enabled || enabled_count < ENABLED_LIMIT {
        return Ok(());
    }
    Err(AppError::Conflict(
        "每种语言最多启用 12 个 SEO 主题词".to_owned(),
    ))
}
