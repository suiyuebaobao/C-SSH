use cloud_domain::{AppError, AppResult, Page, PageQuery};
use cloud_store::{PgPool, Postgres, Transaction};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    Announcement,
    model::{AnnouncementRow, CurrentPublication, PublicationStateRow, ValidatedAnnouncement},
};

const COLUMNS: &str = "id, title_zh_cn, body_zh_cn, title_en, body_en, status, revision, \
created_by, updated_by, published_at, hidden_at, created_at, updated_at";

pub(crate) async fn list(pool: &PgPool, page: PageQuery) -> AppResult<Page<Announcement>> {
    let page = page.normalized();
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM global_announcements")
        .fetch_one(pool)
        .await
        .map_err(read_error)?;
    let query = format!(
        "SELECT {COLUMNS} FROM global_announcements \
         ORDER BY CASE status WHEN 'published' THEN 0 WHEN 'draft' THEN 1 ELSE 2 END, \
         updated_at DESC, id DESC LIMIT $1 OFFSET $2"
    );
    let rows = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(read_error)?;
    let items = rows
        .into_iter()
        .map(Announcement::try_from)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn get(pool: &PgPool, id: Uuid) -> AppResult<Announcement> {
    let query = format!("SELECT {COLUMNS} FROM global_announcements WHERE id = $1");
    let row = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(read_error)?
        .ok_or_else(|| AppError::NotFound("公告不存在".to_owned()))?;
    Announcement::try_from(row)
}

pub(crate) async fn current(pool: &PgPool) -> AppResult<CurrentPublication> {
    let mut transaction = pool.begin().await.map_err(read_error)?;
    let state = sqlx::query_as::<_, PublicationStateRow>(
        "SELECT public_revision, current_announcement_id \
         FROM global_announcement_publication_state \
         WHERE singleton = TRUE FOR SHARE",
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(read_error)?;
    let announcement = if let Some(id) = state.current_announcement_id {
        let query = format!(
            "SELECT {COLUMNS} FROM global_announcements \
             WHERE id = $1 AND status = 'published'"
        );
        let row = sqlx::query_as::<_, AnnouncementRow>(&query)
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(read_error)?
            .ok_or_else(|| AppError::Storage("公告公开状态与已发布内容不一致".to_owned()))?;
        Some(Announcement::try_from(row)?)
    } else {
        None
    };
    transaction.commit().await.map_err(read_error)?;
    Ok(CurrentPublication {
        public_revision: state.public_revision,
        announcement,
    })
}

pub(crate) async fn lock(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> AppResult<Announcement> {
    let query = format!("SELECT {COLUMNS} FROM global_announcements WHERE id = $1 FOR UPDATE");
    let row = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(|| AppError::NotFound("公告不存在".to_owned()))?;
    Announcement::try_from(row)
}

pub(crate) async fn lock_publication(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<PublicationStateRow> {
    sqlx::query_as::<_, PublicationStateRow>(
        "SELECT public_revision, current_announcement_id \
         FROM global_announcement_publication_state \
         WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)
}

pub(crate) async fn create(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    value: &ValidatedAnnouncement,
) -> AppResult<Announcement> {
    let query = format!(
        "INSERT INTO global_announcements \
         (id, title_zh_cn, body_zh_cn, title_en, body_en, status, revision, \
          created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, 'draft', 1, $6, $6) RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(Uuid::now_v7())
        .bind(&value.title_zh_cn)
        .bind(&value.body_zh_cn)
        .bind(&value.title_en)
        .bind(&value.body_en)
        .bind(actor_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(write_error)?;
    Announcement::try_from(row)
}

pub(crate) async fn replace_draft(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
    value: &ValidatedAnnouncement,
) -> AppResult<Announcement> {
    let query = format!(
        "UPDATE global_announcements SET title_zh_cn = $4, body_zh_cn = $5, \
         title_en = $6, body_en = $7, revision = revision + 1, updated_by = $3, \
         updated_at = now() WHERE id = $1 AND revision = $2 AND status = 'draft' \
         RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(id)
        .bind(expected_revision)
        .bind(actor_id)
        .bind(&value.title_zh_cn)
        .bind(&value.body_zh_cn)
        .bind(&value.title_en)
        .bind(&value.body_en)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(changed)?;
    Announcement::try_from(row)
}

pub(crate) async fn hide_current(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    id: Uuid,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE global_announcements SET status = 'hidden', revision = revision + 1, \
         updated_by = $1, hidden_at = now(), updated_at = now() \
         WHERE id = $2 AND status = 'published'",
    )
    .bind(actor_id)
    .bind(id)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    if result.rows_affected() != 1 {
        return Err(AppError::Conflict(
            "当前已发布公告状态已变化，请刷新后重试".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) async fn advance_publication(
    transaction: &mut Transaction<'_, Postgres>,
    expected_revision: i64,
    current_announcement_id: Option<Uuid>,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        "UPDATE global_announcement_publication_state \
         SET public_revision = public_revision + 1, current_announcement_id = $2, \
             updated_at = now() \
         WHERE singleton = TRUE AND public_revision = $1 \
         RETURNING public_revision",
    )
    .bind(expected_revision)
    .bind(current_announcement_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::Conflict("公告公开版本已变化，请刷新后重试".to_owned()))
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    action: &str,
    announcement_id: Uuid,
    status: &str,
    revision: i64,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, $3, 'announcements', $4, 'success', $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(action)
    .bind(announcement_id.to_string())
    .bind(request_id)
    .bind(audit_details(announcement_id, status, revision))
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存公告审计失败".to_owned()))?;
    Ok(())
}

pub(crate) fn audit_details(announcement_id: Uuid, status: &str, revision: i64) -> Value {
    json!({
        "announcement_id": announcement_id,
        "status": status,
        "revision": revision
    })
}

pub(crate) async fn publish(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<Announcement> {
    transition(
        transaction,
        actor_id,
        id,
        expected_revision,
        "draft",
        "published",
        "published_at = now(), hidden_at = NULL",
    )
    .await
}

pub(crate) async fn hide(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<Announcement> {
    transition(
        transaction,
        actor_id,
        id,
        expected_revision,
        "published",
        "hidden",
        "hidden_at = now()",
    )
    .await
}

pub(crate) async fn delete_draft(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<()> {
    let result = sqlx::query(
        "DELETE FROM global_announcements \
         WHERE id = $1 AND revision = $2 AND status = 'draft'",
    )
    .bind(id)
    .bind(expected_revision)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    if result.rows_affected() != 1 {
        return Err(changed());
    }
    Ok(())
}

async fn transition(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
    from: &str,
    to: &str,
    timestamps: &str,
) -> AppResult<Announcement> {
    let query = format!(
        "UPDATE global_announcements SET status = $4, revision = revision + 1, \
         updated_by = $3, {timestamps}, updated_at = now() \
         WHERE id = $1 AND revision = $2 AND status = $5 RETURNING {COLUMNS}"
    );
    let row = sqlx::query_as::<_, AnnouncementRow>(&query)
        .bind(id)
        .bind(expected_revision)
        .bind(actor_id)
        .bind(to)
        .bind(from)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(write_error)?
        .ok_or_else(changed)?;
    Announcement::try_from(row)
}

fn changed() -> AppError {
    AppError::Conflict("公告状态或版本已变化，请刷新后重试".to_owned())
}

fn read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取公告失败".to_owned())
}

fn write_error(error: sqlx::Error) -> AppError {
    if matches!(&error, sqlx::Error::Database(database) if database.is_unique_violation()) {
        AppError::Conflict("当前已有已发布公告".to_owned())
    } else {
        AppError::Storage("保存公告失败".to_owned())
    }
}
