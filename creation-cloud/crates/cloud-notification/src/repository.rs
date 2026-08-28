//! 封装通知投递、账号分页、回执和语义审计的 PostgreSQL 操作。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, Page, PageQuery};
use cloud_store::{PgPool, Postgres, Transaction};
use serde_json::json;
use uuid::Uuid;

use crate::{
    AdminNotification, CreateNotificationInput, NotificationKind, NotificationPriority,
    NotificationReceipt, model::NotificationRow, validation::ValidatedListQuery,
};

const COLUMNS: &str = "notification.id, notification.account_id, notification.revision, \
notification.kind, notification.priority, notification.code, notification.resource_id, \
notification.published_at, notification.expires_at, notification.created_by";

pub(crate) async fn list_account(
    pool: &PgPool,
    account_id: Uuid,
    query: &ValidatedListQuery,
) -> AppResult<Vec<NotificationRow>> {
    let (cursor_rank, cursor_at, cursor_id) =
        query.cursor.map_or((None, None, None), |(rank, at, id)| {
            (Some(rank), Some(at), Some(id))
        });
    let sql = format!(
        "SELECT {COLUMNS}, receipt.read_at FROM account_notifications AS notification \
         LEFT JOIN account_notification_receipts AS receipt \
           ON receipt.notification_id = notification.id \
          AND receipt.account_id = notification.account_id \
          AND receipt.notification_revision = notification.revision \
         WHERE notification.account_id = $1 \
           AND notification.published_at <= now() \
           AND (notification.expires_at IS NULL OR notification.expires_at > now()) \
           AND ($2::smallint IS NULL OR \
             (CASE notification.priority WHEN 'critical' THEN 3 WHEN 'important' THEN 2 ELSE 1 END, \
              notification.published_at, notification.id) < ($2, $3, $4)) \
         ORDER BY CASE notification.priority WHEN 'critical' THEN 3 \
                   WHEN 'important' THEN 2 ELSE 1 END DESC, \
                  notification.published_at DESC, notification.id DESC LIMIT $5"
    );
    sqlx::query_as::<_, NotificationRow>(&sql)
        .bind(account_id)
        .bind(cursor_rank)
        .bind(cursor_at)
        .bind(cursor_id)
        .bind(query.limit + 1)
        .fetch_all(pool)
        .await
        .map_err(read_error)
}

pub(crate) async fn unread_count(pool: &PgPool, account_id: Uuid) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_notifications AS notification \
         WHERE notification.account_id = $1 AND notification.published_at <= now() \
           AND (notification.expires_at IS NULL OR notification.expires_at > now()) \
           AND NOT EXISTS (SELECT 1 FROM account_notification_receipts AS receipt \
             WHERE receipt.notification_id = notification.id \
               AND receipt.account_id = notification.account_id \
               AND receipt.notification_revision = notification.revision)",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(read_error)
}

pub(crate) async fn receipt(
    pool: &PgPool,
    account_id: Uuid,
    notification_id: Uuid,
    revision: i64,
) -> AppResult<NotificationReceipt> {
    let row = sqlx::query_as::<_, (Uuid, i64, DateTime<Utc>)>(
        "INSERT INTO account_notification_receipts \
         (notification_id, account_id, notification_revision) \
         SELECT id, account_id, revision FROM account_notifications \
         WHERE id = $1 AND account_id = $2 AND revision = $3 \
           AND published_at <= now() AND (expires_at IS NULL OR expires_at > now()) \
         ON CONFLICT (notification_id, account_id) DO UPDATE \
         SET notification_revision = EXCLUDED.notification_revision, \
             read_at = account_notification_receipts.read_at \
         RETURNING notification_id, notification_revision, read_at",
    )
    .bind(notification_id)
    .bind(account_id)
    .bind(revision)
    .fetch_optional(pool)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::NotFound("通知不存在、已过期或修订已变化".into()))?;
    Ok(NotificationReceipt {
        notification_id: row.0,
        revision: row.1,
        read_at: row.2,
    })
}

pub(crate) async fn list_admin(
    pool: &PgPool,
    page: PageQuery,
) -> AppResult<Page<AdminNotification>> {
    let page = page.normalized();
    let total = sqlx::query_scalar("SELECT COUNT(*) FROM account_notifications")
        .fetch_one(pool)
        .await
        .map_err(read_error)?;
    let sql = format!(
        "SELECT {COLUMNS}, NULL::timestamptz AS read_at \
         FROM account_notifications AS notification \
         ORDER BY notification.published_at DESC, notification.id DESC LIMIT $1 OFFSET $2"
    );
    let rows = sqlx::query_as::<_, NotificationRow>(&sql)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(read_error)?;
    let items = rows
        .into_iter()
        .map(admin_projection)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn create(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    input: &CreateNotificationInput,
) -> AppResult<AdminNotification> {
    let account_active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND status = 'active')",
    )
    .bind(input.account_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)?;
    if !account_active {
        return Err(AppError::NotFound("目标活动账号不存在".into()));
    }
    let id = Uuid::now_v7();
    let row = sqlx::query_as::<_, NotificationRow>(
        "INSERT INTO account_notifications \
         (id, account_id, kind, priority, code, resource_id, expires_at, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, account_id, revision, kind, priority, code, resource_id, \
                   published_at, expires_at, created_by, NULL::timestamptz AS read_at",
    )
    .bind(id)
    .bind(input.account_id)
    .bind(input.kind.as_str())
    .bind(input.priority.as_str())
    .bind(&input.code)
    .bind(input.resource_id)
    .bind(input.expires_at)
    .bind(actor_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)?;
    let record = admin_projection(row)?;
    audit(transaction, actor_id, &record).await?;
    Ok(record)
}

async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    record: &AdminNotification,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = json!({
        "notification_id": record.id,
        "account_id": record.account_id,
        "kind": record.kind.as_str(),
        "priority": record.priority.as_str(),
        "code": record.code,
        "resource_id": record.resource_id,
        "revision": record.revision
    });
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'notification.created', 'account_notification', $3, 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(record.id.to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存通知审计失败".into()))?;
    Ok(())
}

fn admin_projection(row: NotificationRow) -> AppResult<AdminNotification> {
    Ok(AdminNotification {
        id: row.id,
        account_id: row.account_id,
        revision: row.revision,
        kind: parse_kind(&row.kind)?,
        priority: parse_priority(&row.priority)?,
        code: row.code,
        resource_id: row.resource_id,
        published_at: row.published_at,
        expires_at: row.expires_at,
        created_by: row.created_by,
    })
}

pub(crate) fn parse_kind(value: &str) -> AppResult<NotificationKind> {
    match value {
        "account_security" => Ok(NotificationKind::AccountSecurity),
        "sync" => Ok(NotificationKind::Sync),
        _ => Err(AppError::Storage("数据库中的通知类型无效".into())),
    }
}

pub(crate) fn parse_priority(value: &str) -> AppResult<NotificationPriority> {
    match value {
        "normal" => Ok(NotificationPriority::Normal),
        "important" => Ok(NotificationPriority::Important),
        "critical" => Ok(NotificationPriority::Critical),
        _ => Err(AppError::Storage("数据库中的通知优先级无效".into())),
    }
}

fn read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取账号通知失败".into())
}

fn write_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("保存账号通知失败".into())
}
