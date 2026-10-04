//! 在PostgreSQL事务内执行教学记录CAS及最小语义审计。

use cloud_domain::{AdminActor, AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use serde_json::json;
use uuid::Uuid;

use crate::{
    model::{Metadata, Record},
    multipart::Upload,
};

const COLUMNS: &str = "id, locale, title, description, sort_order, version_note, state, revision, \
    video_bytes, video_sha256, poster_bytes, poster_sha256, updated_at, purged_at";

pub(crate) async fn list(
    pool: &PgPool,
    locale: Option<&str>,
    offset: u32,
    public: bool,
) -> AppResult<Vec<Record>> {
    let filter = if public {
        "state = 'published'"
    } else {
        "(state <> 'deleted' OR purged_at IS NULL)"
    };
    sqlx::query_as(&format!("SELECT {COLUMNS} FROM tutorial_videos WHERE {filter} \
        AND ($1::text IS NULL OR locale = $1) ORDER BY sort_order, created_at, id LIMIT 100 OFFSET $2"))
        .bind(locale).bind(i64::from(offset)).fetch_all(pool).await.map_err(error)
}

pub(crate) async fn get(pool: &PgPool, id: Uuid) -> AppResult<Record> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM tutorial_videos WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(error)?
    .ok_or_else(|| AppError::NotFound("教学视频不存在".into()))
}

pub(crate) async fn lock(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> AppResult<Record> {
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM tutorial_videos WHERE id = $1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(error)?
    .ok_or_else(|| AppError::NotFound("教学视频不存在".into()))
}

pub(crate) async fn create(
    tx: &mut Transaction<'_, Postgres>,
    actor: AdminActor,
    id: Uuid,
    upload: &Upload,
) -> AppResult<Record> {
    let m = &upload.metadata;
    sqlx::query_as(&format!("INSERT INTO tutorial_videos \
        (id, locale, title, description, sort_order, version_note, video_bytes, video_sha256, poster_bytes, poster_sha256, created_by) \
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING {COLUMNS}"))
        .bind(id).bind(&m.locale).bind(&m.title).bind(&m.description).bind(m.sort_order)
        .bind(&m.version_note).bind(upload.video.bytes as i64).bind(&upload.video.sha256)
        .bind(upload.poster.bytes as i64).bind(&upload.poster.sha256).bind(actor.account_id())
        .fetch_one(&mut **tx).await.map_err(error)
}

pub(crate) async fn update(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    m: &Metadata,
) -> AppResult<Record> {
    sqlx::query_as(&format!("UPDATE tutorial_videos SET locale=$2, title=$3, description=$4, \
        sort_order=$5, version_note=$6, revision=revision+1, updated_at=now() WHERE id=$1 RETURNING {COLUMNS}"))
        .bind(id).bind(&m.locale).bind(&m.title).bind(&m.description).bind(m.sort_order).bind(&m.version_note)
        .fetch_one(&mut **tx).await.map_err(error)
}

pub(crate) async fn transition(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    state: &str,
) -> AppResult<Record> {
    sqlx::query_as(&format!("UPDATE tutorial_videos SET state=$2, revision=revision+1, updated_at=now(), \
        description=CASE WHEN $2='deleted' THEN '' ELSE description END, \
        version_note=CASE WHEN $2='deleted' THEN '' ELSE version_note END WHERE id=$1 RETURNING {COLUMNS}"))
        .bind(id).bind(state).fetch_one(&mut **tx).await.map_err(error)
}

pub(crate) async fn purged(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE tutorial_videos SET purged_at=now() WHERE id=$1 AND state='deleted'")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(error)?;
    Ok(())
}

pub(crate) async fn audit(
    tx: &mut Transaction<'_, Postgres>,
    actor: AdminActor,
    action: &str,
    row: &Record,
    request_id: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_events (id, actor_account_id, action, resource_kind, resource_id, outcome, request_id, details) \
        VALUES ($1,$2,$3,'tutorial_video',$4,'success',$5,$6)")
        .bind(Uuid::now_v7()).bind(actor.account_id()).bind(action).bind(row.id.to_string()).bind(request_id)
        .bind(json!({"video_id":row.id,"revision":row.revision,"state":row.state,"locale":row.locale}))
        .execute(&mut **tx).await.map_err(error)?;
    Ok(())
}

pub(crate) fn error(_: sqlx::Error) -> AppError {
    AppError::Storage("教学视频资料操作失败".into())
}
