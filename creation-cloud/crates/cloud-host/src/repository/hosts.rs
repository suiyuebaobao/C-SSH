//! 读取账号主机元数据；此路径只投影密文存在性而不返回密文正文。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult, Page, PageQuery};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::HostView;

use super::storage;

#[derive(Clone, Debug, FromRow)]
struct HostViewRow {
    pub id: Uuid,
    pub secret_present: bool,
    pub metadata_encrypted: bool,
    pub source_device_id: Uuid,
    pub revision: i64,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HostViewRow {
    fn view(self) -> AppResult<HostView> {
        Ok(HostView {
            id: self.id,
            revision: self.revision,
            source_device_id: self.source_device_id,
            deleted: self.is_deleted,
            secret_present: self.secret_present,
            metadata_encrypted: self.metadata_encrypted,
            host_metadata_migration_required: !self.metadata_encrypted,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct HostRow {
    pub address: Option<String>,
    pub port: Option<i32>,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub status: Option<String>,
    pub ciphertext: Option<Vec<u8>>,
    pub revision: i64,
    pub is_deleted: bool,
    pub metadata_encrypted: bool,
}

pub(crate) async fn count(pool: &PgPool, account_id: Uuid) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*)::BIGINT
         FROM cloud_hosts
         WHERE account_id = $1 AND NOT is_deleted",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(storage)
}

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<HostView>> {
    let page = page.normalized();
    let total = count(pool, account_id).await?;
    let rows = sqlx::query_as::<_, HostViewRow>(
        "SELECT id, (ciphertext IS NOT NULL) AS secret_present, metadata_encrypted,
                source_device_id, revision, is_deleted, created_at, updated_at
         FROM cloud_hosts
         WHERE account_id = $1 AND NOT is_deleted
         ORDER BY updated_at DESC, id
         LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(storage)?;
    let items = rows
        .into_iter()
        .map(HostViewRow::view)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn get(pool: &PgPool, account_id: Uuid, host_id: Uuid) -> AppResult<HostView> {
    let row = sqlx::query_as::<_, HostViewRow>(
        "SELECT id, (ciphertext IS NOT NULL) AS secret_present, metadata_encrypted,
                source_device_id, revision, is_deleted, created_at, updated_at
         FROM cloud_hosts
         WHERE account_id = $1 AND id = $2 AND NOT is_deleted",
    )
    .bind(account_id)
    .bind(host_id)
    .fetch_optional(pool)
    .await
    .map_err(storage)?
    .ok_or_else(|| AppError::NotFound("host was not found".to_owned()))?;
    row.view()
}

pub(crate) async fn lock_current(
    tx: &mut super::DbTransaction<'_>,
    account_id: Uuid,
    host_id: Uuid,
) -> AppResult<Option<HostRow>> {
    sqlx::query_as::<_, HostRow>(
        "SELECT address, port, name, platform, tags, status, ciphertext,
                revision, is_deleted, metadata_encrypted
         FROM cloud_hosts
         WHERE account_id = $1 AND id = $2
         FOR UPDATE",
    )
    .bind(account_id)
    .bind(host_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)
}
