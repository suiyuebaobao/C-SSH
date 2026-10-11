//! 以每版本独立计数器串行分配修订，CAS、创建及语义审计同事务完成。

use crate::{
    CreateRevisionInput, Release, ReleaseStatus,
    model::ReleaseRow,
    repository::{map_read_error, map_transaction_error, map_write_error},
};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

pub(crate) async fn execute(
    pool: &PgPool,
    actor_id: Uuid,
    source_id: Uuid,
    input: &CreateRevisionInput,
) -> AppResult<Release> {
    let source = super::get::execute(pool, source_id).await?;
    let mut transaction = pool.begin().await.map_err(map_transaction_error)?;
    sqlx::query("INSERT INTO release_revision_counters (version, last_revision) VALUES ($1, 1) ON CONFLICT DO NOTHING")
        .bind(&source.version).execute(&mut *transaction).await.map_err(map_transaction_error)?;
    let last: i32 = sqlx::query_scalar(
        "SELECT last_revision FROM release_revision_counters WHERE version = $1 FOR UPDATE",
    )
    .bind(&source.version)
    .fetch_one(&mut *transaction)
    .await
    .map_err(map_read_error)?;
    let source = super::lock::execute(&mut transaction, source_id).await?;
    if source.status != ReleaseStatus::Published
        || source.updated_at != input.expected_updated_at
        || last != input.expected_latest_revision
    {
        return Err(AppError::Conflict(
            "源发布或最新修订已变化，请刷新后重试".into(),
        ));
    }
    let revision = last
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict("修订号已达上限".into()))?;
    let row = sqlx::query_as::<_, ReleaseRow>(
        "INSERT INTO releases (id, version, revision, channel, status, title_zh, title_en, notes_zh, notes_en) \
         VALUES ($1, $2, $3, $4, 'draft', $5, $6, $7, $8) \
         RETURNING id, version, revision,
        GREATEST(revision, COALESCE((SELECT counter.last_revision FROM release_revision_counters AS counter WHERE counter.version = releases.version), 1)) AS latest_revision, channel, status, title_zh, title_en, notes_zh, notes_en, published_at, created_at, updated_at")
        .bind(Uuid::now_v7()).bind(&source.version).bind(revision).bind(source.channel.as_str())
        .bind(&source.title_zh).bind(&source.title_en).bind(&source.notes_zh).bind(&source.notes_en)
        .fetch_one(&mut *transaction).await.map_err(|error| map_write_error(error, "修订创建冲突"))?;
    sqlx::query("UPDATE release_revision_counters SET last_revision = $2 WHERE version = $1")
        .bind(&source.version)
        .bind(revision)
        .execute(&mut *transaction)
        .await
        .map_err(map_transaction_error)?;
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query("INSERT INTO audit_events (id, actor_account_id, action, resource_kind, resource_id, outcome, request_id, details) \
        VALUES ($1, $2, 'release.revision.create', 'release', $3, 'success', $4, $5)")
        .bind(Uuid::now_v7()).bind(actor_id).bind(row.id.to_string()).bind(request_id)
        .bind(serde_json::json!({"source_release_id": source_id, "version": source.version, "revision": revision}))
        .execute(&mut *transaction).await.map_err(map_transaction_error)?;
    let release = Release::try_from(row)?;
    transaction.commit().await.map_err(map_transaction_error)?;
    Ok(release)
}
