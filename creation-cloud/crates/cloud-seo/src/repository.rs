use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{CreateSeoTopicInput, SeoLocale, SeoTopic, UpdateSeoTopicInput, model::SeoTopicRow};

const COLUMNS: &str = "id, locale, phrase, sort_order, enabled, created_by, created_at, updated_at";

pub(crate) async fn lock_writes(transaction: &mut Transaction<'_, Postgres>) -> AppResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('creation-cloud:seo-topics', 0))")
        .execute(&mut **transaction)
        .await
        .map_err(transaction_error)?;
    Ok(())
}

pub(crate) async fn enabled_count(
    transaction: &mut Transaction<'_, Postgres>,
    locale: SeoLocale,
    excluding: Option<Uuid>,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM seo_topics WHERE locale = $1 AND enabled AND ($2::uuid IS NULL OR id <> $2)",
    )
    .bind(locale.as_str())
    .bind(excluding)
    .fetch_one(&mut **transaction)
    .await
    .map_err(read_error)
}

pub(crate) async fn create(
    transaction: &mut Transaction<'_, Postgres>,
    created_by: Uuid,
    input: &CreateSeoTopicInput,
) -> AppResult<SeoTopic> {
    let sql = format!(
        "INSERT INTO seo_topics (id, locale, phrase, sort_order, enabled, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, SeoTopicRow>(&sql)
        .bind(Uuid::now_v7())
        .bind(input.locale.as_str())
        .bind(&input.phrase)
        .bind(input.sort_order)
        .bind(input.enabled)
        .bind(created_by)
        .fetch_one(&mut **transaction)
        .await
        .map_err(write_error)?;
    SeoTopic::try_from(row)
}

pub(crate) async fn list(pool: &PgPool) -> AppResult<Vec<SeoTopic>> {
    let sql =
        format!("SELECT {COLUMNS} FROM seo_topics ORDER BY locale, sort_order, created_at, id");
    rows(sqlx::query_as::<_, SeoTopicRow>(&sql).fetch_all(pool).await)
}

pub(crate) async fn public_list(pool: &PgPool, locale: SeoLocale) -> AppResult<Vec<SeoTopic>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM seo_topics \
         WHERE locale = $1 AND enabled ORDER BY sort_order, created_at, id"
    );
    rows(
        sqlx::query_as::<_, SeoTopicRow>(&sql)
            .bind(locale.as_str())
            .fetch_all(pool)
            .await,
    )
}

pub(crate) async fn lock_one(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> AppResult<SeoTopic> {
    let sql = format!("SELECT {COLUMNS} FROM seo_topics WHERE id = $1 FOR UPDATE");
    let row = sqlx::query_as::<_, SeoTopicRow>(&sql)
        .bind(id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(read_error)?
        .ok_or_else(|| AppError::NotFound("SEO 主题词不存在".to_owned()))?;
    SeoTopic::try_from(row)
}

pub(crate) async fn update(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    input: &UpdateSeoTopicInput,
) -> AppResult<SeoTopic> {
    let sql = format!(
        "UPDATE seo_topics SET locale = COALESCE($2, locale), \
         phrase = COALESCE($3, phrase), sort_order = COALESCE($4, sort_order), \
         enabled = COALESCE($5, enabled) WHERE id = $1 RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, SeoTopicRow>(&sql)
        .bind(id)
        .bind(input.locale.map(SeoLocale::as_str))
        .bind(input.phrase.as_deref())
        .bind(input.sort_order)
        .bind(input.enabled)
        .fetch_one(&mut **transaction)
        .await
        .map_err(write_error)?;
    SeoTopic::try_from(row)
}

pub(crate) async fn delete(transaction: &mut Transaction<'_, Postgres>, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM seo_topics WHERE id = $1")
        .bind(id)
        .execute(&mut **transaction)
        .await
        .map_err(write_error)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("SEO 主题词不存在".to_owned()));
    }
    Ok(())
}

fn rows(result: Result<Vec<SeoTopicRow>, sqlx::Error>) -> AppResult<Vec<SeoTopic>> {
    result
        .map_err(read_error)?
        .into_iter()
        .map(SeoTopic::try_from)
        .collect()
}

fn read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取 SEO 主题词失败".to_owned())
}

fn write_error(error: sqlx::Error) -> AppError {
    if error
        .as_database_error()
        .and_then(|database| database.code())
        .is_some_and(|code| matches!(code.as_ref(), "23503" | "23505" | "23514" | "P0001"))
    {
        return AppError::Conflict("SEO 主题词与现有数据冲突".to_owned());
    }
    AppError::Storage("写入 SEO 主题词失败".to_owned())
}

pub(crate) fn transaction_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("执行 SEO 主题词事务失败".to_owned())
}
