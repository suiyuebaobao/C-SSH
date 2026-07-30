//! Administrator projection of explicit upload and download synchronization.

use chrono::{DateTime, Utc};
use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{AdminSyncDirection, AdminSyncRecord};

use super::{invalid_stored_value, storage};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

#[derive(FromRow)]
struct SyncRecordRow {
    record_id: String,
    direction: String,
    device_id: Uuid,
    device_name: String,
    device_platform: String,
    outcome: String,
    revision: i64,
    changed_count: i32,
    occurred_at: DateTime<Utc>,
}

pub(crate) const RECORDS_SQL: &str = r#"
    SELECT client_mutation_id::TEXT AS record_id,
           'upload'::TEXT AS direction,
           source_device_id AS device_id,
           devices.name AS device_name,
           devices.platform AS device_platform,
           outcome,
           result_revision AS revision,
           changed_count,
           cloud_host_mutations.created_at AS occurred_at
    FROM cloud_host_mutations
    JOIN devices
      ON devices.account_id = cloud_host_mutations.account_id
     AND devices.id = cloud_host_mutations.source_device_id
    WHERE cloud_host_mutations.account_id = $1

    UNION ALL

    SELECT concat('download:', cloud_host_device_checkpoints.device_id::TEXT) AS record_id,
           'download'::TEXT AS direction,
           cloud_host_device_checkpoints.device_id,
           devices.name AS device_name,
           devices.platform AS device_platform,
           'acknowledged'::TEXT AS outcome,
           cloud_host_device_checkpoints.acknowledged_revision AS revision,
           0::INTEGER AS changed_count,
           cloud_host_device_checkpoints.last_manual_sync_at AS occurred_at
    FROM cloud_host_device_checkpoints
    JOIN devices
      ON devices.account_id = cloud_host_device_checkpoints.account_id
     AND devices.id = cloud_host_device_checkpoints.device_id
    WHERE cloud_host_device_checkpoints.account_id = $1
"#;

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<AdminSyncRecord>> {
    let page = page.normalized();
    let count_sql = format!("SELECT count(*)::BIGINT AS total FROM ({RECORDS_SQL}) records");
    let list_sql = format!(
        "SELECT * FROM ({RECORDS_SQL}) records
         ORDER BY occurred_at DESC, record_id DESC
         LIMIT $2 OFFSET $3"
    );
    let count = sqlx::query_as::<_, CountRow>(&count_sql)
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(storage)?;
    let rows = sqlx::query_as::<_, SyncRecordRow>(&list_sql)
        .bind(account_id)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(storage)?;
    let items = rows
        .into_iter()
        .map(|row| {
            Ok(AdminSyncRecord {
                record_id: row.record_id,
                direction: AdminSyncDirection::parse(&row.direction)
                    .ok_or_else(invalid_stored_value)?,
                device_id: row.device_id,
                device_name: row.device_name,
                device_platform: row.device_platform,
                outcome: row.outcome,
                revision: row.revision,
                changed_count: row.changed_count,
                occurred_at: row.occurred_at,
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total: count.total,
    })
}

#[cfg(test)]
mod tests {
    use super::RECORDS_SQL;

    #[test]
    fn admin_projection_contains_only_safe_sync_metadata() {
        assert!(RECORDS_SQL.contains("'upload'::TEXT"));
        assert!(RECORDS_SQL.contains("'download'::TEXT"));
        assert!(RECORDS_SQL.contains("last_manual_sync_at"));
        assert!(!RECORDS_SQL.contains("request_hash"));
        assert!(!RECORDS_SQL.contains("ciphertext"));
    }
}
