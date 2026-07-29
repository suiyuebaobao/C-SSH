use cloud_domain::{AppError, AppResult};
use cloud_site::Locale;
use cloud_store::{PgPool, Postgres, Transaction};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    SiteContentDocumentKey, SiteContentPayload, SiteContentRevision, SiteContentState,
    model::SiteContentRow,
};

const COLUMNS: &str = "id, document_key, locale, state, revision, content_json, created_by, \
published_at, revoked_at, created_at, updated_at";

pub(crate) async fn list(
    pool: &PgPool,
    key: Option<SiteContentDocumentKey>,
    locale: Option<Locale>,
) -> AppResult<Vec<SiteContentRevision>> {
    let query = format!(
        "SELECT {COLUMNS} FROM site_content_revisions \
         WHERE ($1::TEXT IS NULL OR document_key = $1) \
           AND ($2::TEXT IS NULL OR locale = $2) \
         ORDER BY created_at DESC, id DESC LIMIT 200"
    );
    let rows = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(key.map(SiteContentDocumentKey::as_str))
        .bind(locale.map(Locale::code))
        .fetch_all(pool)
        .await
        .map_err(read_error)?;
    rows.into_iter()
        .map(SiteContentRevision::try_from)
        .collect()
}

pub(crate) async fn get(pool: &PgPool, id: Uuid) -> AppResult<SiteContentRevision> {
    let query = format!("SELECT {COLUMNS} FROM site_content_revisions WHERE id = $1");
    let row = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(read_error)?
        .ok_or_else(|| AppError::NotFound("站点内容记录不存在".into()))?;
    SiteContentRevision::try_from(row)
}

pub(crate) async fn published(
    pool: &PgPool,
    key: SiteContentDocumentKey,
    locale: Locale,
) -> AppResult<Option<SiteContentRevision>> {
    let query = format!(
        "SELECT {COLUMNS} FROM site_content_revisions \
         WHERE document_key = $1 AND locale = $2 AND state = 'published'"
    );
    sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(key.as_str())
        .bind(locale.code())
        .fetch_optional(pool)
        .await
        .map_err(read_error)?
        .map(SiteContentRevision::try_from)
        .transpose()
}

pub(crate) async fn has_publication_history(
    pool: &PgPool,
    key: SiteContentDocumentKey,
    locale: Locale,
) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM site_content_revisions \
         WHERE document_key = $1 AND locale = $2 AND published_at IS NOT NULL)",
    )
    .bind(key.as_str())
    .bind(locale.code())
    .fetch_one(pool)
    .await
    .map_err(read_error)
}

pub(crate) async fn lock(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> AppResult<SiteContentRevision> {
    let query = format!("SELECT {COLUMNS} FROM site_content_revisions WHERE id = $1 FOR UPDATE");
    let row = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(|| AppError::NotFound("站点内容记录不存在".into()))?;
    SiteContentRevision::try_from(row)
}

pub(crate) async fn lock_scope(
    transaction: &mut Transaction<'_, Postgres>,
    key: SiteContentDocumentKey,
    locale: Locale,
) -> AppResult<()> {
    let scope = format!("site-content:{}:{}", key.as_str(), locale.code());
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(scope)
        .execute(&mut **transaction)
        .await
        .map_err(write_error)?;
    Ok(())
}

pub(crate) async fn published_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    key: SiteContentDocumentKey,
    locale: Locale,
) -> AppResult<Option<SiteContentRevision>> {
    let query = format!(
        "SELECT {COLUMNS} FROM site_content_revisions \
         WHERE document_key = $1 AND locale = $2 AND state = 'published' FOR UPDATE"
    );
    sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(key.as_str())
        .bind(locale.code())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .map(SiteContentRevision::try_from)
        .transpose()
}

pub(crate) async fn create_draft(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    key: SiteContentDocumentKey,
    locale: Locale,
    content: &SiteContentPayload,
) -> AppResult<SiteContentRevision> {
    insert(
        transaction,
        actor_id,
        key,
        locale,
        SiteContentState::Draft,
        content,
    )
    .await
}

pub(crate) async fn update_draft(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
    content: &SiteContentPayload,
) -> AppResult<SiteContentRevision> {
    let value = serialize(content)?;
    let query = format!(
        "UPDATE site_content_revisions SET content_json = $3, revision = revision + 1, \
         updated_at = now() WHERE id = $1 AND state = 'draft' AND revision = $2 \
         RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(id)
        .bind(expected_revision)
        .bind(value)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(|| AppError::Conflict("草稿版本已变化，请刷新后重试".into()))?;
    SiteContentRevision::try_from(row)
}

pub(crate) async fn revoke_current(
    transaction: &mut Transaction<'_, Postgres>,
    key: SiteContentDocumentKey,
    locale: Locale,
    excluding: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE site_content_revisions SET state = 'revoked', revoked_at = now(), \
         updated_at = now() WHERE document_key = $1 AND locale = $2 \
         AND state = 'published' AND ($3::UUID IS NULL OR id <> $3)",
    )
    .bind(key.as_str())
    .bind(locale.code())
    .bind(excluding)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    Ok(())
}

pub(crate) async fn publish(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<SiteContentRevision> {
    transition(
        transaction,
        id,
        expected_revision,
        "draft",
        "published",
        "published_at = now(), revoked_at = NULL",
    )
    .await
}

pub(crate) async fn revoke(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<SiteContentRevision> {
    transition(
        transaction,
        id,
        expected_revision,
        "published",
        "revoked",
        "revoked_at = now()",
    )
    .await
}

pub(crate) async fn delete_draft(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<()> {
    let result = sqlx::query(
        "DELETE FROM site_content_revisions \
         WHERE id = $1 AND state = 'draft' AND revision = $2",
    )
    .bind(id)
    .bind(expected_revision)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    if result.rows_affected() != 1 {
        return Err(AppError::Conflict("草稿版本已变化，请刷新后重试".into()));
    }
    Ok(())
}

pub(crate) async fn rollback(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    source: &SiteContentRevision,
) -> AppResult<SiteContentRevision> {
    insert(
        transaction,
        actor_id,
        source.document_key,
        source.locale,
        SiteContentState::Published,
        &source.content,
    )
    .await
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    action: &str,
    record: &SiteContentRevision,
    field_count: usize,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "content_id": record.id,
        "document_key": record.document_key.as_str(),
        "locale": record.locale.code(),
        "revision": record.revision,
        "state": record.state.as_str(),
        "field_count": field_count
    });
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, $3, 'site_content', $4, 'success', $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(action)
    .bind(record.id.to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    Ok(())
}

async fn insert(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    key: SiteContentDocumentKey,
    locale: Locale,
    state: SiteContentState,
    content: &SiteContentPayload,
) -> AppResult<SiteContentRevision> {
    let value = serialize(content)?;
    let query = format!(
        "INSERT INTO site_content_revisions \
         (id, document_key, locale, state, revision, content_json, created_by, published_at) \
         VALUES ($1, $2, $3, $4, 1, $5, $6, \
         CASE WHEN $4 = 'published' THEN now() ELSE NULL END) RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(Uuid::now_v7())
        .bind(key.as_str())
        .bind(locale.code())
        .bind(state.as_str())
        .bind(value)
        .bind(actor_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(write_error)?;
    SiteContentRevision::try_from(row)
}

async fn transition(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
    from: &str,
    to: &str,
    timestamps: &str,
) -> AppResult<SiteContentRevision> {
    let query = format!(
        "UPDATE site_content_revisions SET state = $3, {timestamps}, updated_at = now() \
         WHERE id = $1 AND state = $2 AND revision = $4 RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, SiteContentRow>(&query)
        .bind(id)
        .bind(from)
        .bind(to)
        .bind(expected_revision)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(|| AppError::Conflict("站点内容状态或版本已变化".into()))?;
    SiteContentRevision::try_from(row)
}

fn serialize(content: &SiteContentPayload) -> AppResult<Value> {
    serde_json::to_value(content).map_err(|_| AppError::Validation("站点内容无法序列化".into()))
}

fn read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取站点内容失败".into())
}

fn write_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("该语种已有当前发布版本".into())
    } else {
        AppError::Storage("保存站点内容失败".into())
    }
}
