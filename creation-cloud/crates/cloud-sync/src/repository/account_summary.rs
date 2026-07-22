//! 读取账号级同步统计与安全元数据，查询永不选择同步值或冲突正文。

use chrono::{DateTime, Utc};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{AccountSyncSummary, SyncConflictSummary, SyncRecordSummary};

use super::storage;

pub(crate) const COUNTS_SQL: &str = r#"
SELECT COALESCE((SELECT current_revision FROM sync_states WHERE account_id = $1), 0),
       COUNT(*) FILTER (WHERE is_deleted = FALSE),
       COUNT(*) FILTER (WHERE is_deleted = TRUE),
       (SELECT COUNT(*) FROM sync_conflicts
        WHERE account_id = $1 AND resolved_at IS NULL)
FROM sync_records
WHERE account_id = $1
"#;

pub(crate) const RECENT_RECORDS_SQL: &str = r#"
SELECT namespace, record_key, revision, is_deleted, source_device_id, updated_at
FROM sync_records
WHERE account_id = $1
ORDER BY revision DESC, namespace, record_key
LIMIT 20
"#;

pub(crate) const RECENT_CONFLICTS_SQL: &str = r#"
SELECT id, base_revision, current_revision, source_device_id, created_at
FROM sync_conflicts
WHERE account_id = $1 AND resolved_at IS NULL
ORDER BY created_at DESC, id DESC
LIMIT 20
"#;

type CountsRow = (i64, i64, i64, i64);
type RecordRow = (String, String, i64, bool, Option<Uuid>, DateTime<Utc>);
type ConflictRow = (Uuid, i64, i64, Option<Uuid>, DateTime<Utc>);

pub(crate) async fn account_summary(
    pool: &PgPool,
    session: &AuthenticatedSession,
) -> AppResult<AccountSyncSummary> {
    let account_id = session.account_id;
    let counts = sqlx::query_as::<_, CountsRow>(COUNTS_SQL)
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(storage("无法读取同步摘要统计"))?;
    let recent_records = sqlx::query_as::<_, RecordRow>(RECENT_RECORDS_SQL)
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(storage("无法读取最近同步元数据"))?
        .into_iter()
        .map(|row| SyncRecordSummary {
            namespace: row.0,
            key: row.1,
            revision: row.2,
            deleted: row.3,
            source_device_id: row.4,
            updated_at: row.5,
        })
        .collect();
    let recent_conflicts = sqlx::query_as::<_, ConflictRow>(RECENT_CONFLICTS_SQL)
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(storage("无法读取同步冲突元数据"))?
        .into_iter()
        .map(|row| SyncConflictSummary {
            id: row.0,
            base_revision: row.1,
            current_revision: row.2,
            source_device_id: row.3,
            created_at: row.4,
        })
        .collect();
    Ok(AccountSyncSummary {
        current_revision: counts.0,
        active_record_count: counts.1,
        tombstone_count: counts.2,
        unresolved_conflict_count: counts.3,
        recent_records,
        recent_conflicts,
    })
}
