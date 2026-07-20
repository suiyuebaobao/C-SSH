//! 按 revision 稳定读取当前账号的同步记录与墓碑。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use serde_json::Value;
use uuid::Uuid;

use crate::{PullRequest, PullResponse, SyncRecord};

use super::storage;

type RecordRow = (String, String, i64, Option<Value>, bool, DateTime<Utc>);

pub(crate) async fn pull(
    pool: &PgPool,
    account_id: Uuid,
    request: PullRequest,
) -> AppResult<PullResponse> {
    let fetch_limit = i64::from(request.limit) + 1;
    let rows = sqlx::query_as::<_, RecordRow>(
        "SELECT namespace, record_key, revision, value, is_deleted, updated_at \
         FROM sync_records WHERE account_id = $1 AND revision > $2 \
         ORDER BY revision ASC, namespace ASC, record_key ASC LIMIT $3",
    )
    .bind(account_id)
    .bind(request.since_revision)
    .bind(fetch_limit)
    .fetch_all(pool)
    .await
    .map_err(storage("无法拉取同步记录"))?;

    let has_more = rows.len() > request.limit as usize;
    let records = rows
        .into_iter()
        .take(request.limit as usize)
        .map(|row| SyncRecord {
            namespace: row.0,
            key: row.1,
            revision: row.2,
            value: row.3,
            deleted: row.4,
            updated_at: row.5,
        })
        .collect::<Vec<_>>();
    let next_revision = records
        .last()
        .map_or(request.since_revision, |record| record.revision);
    Ok(PullResponse {
        records,
        next_revision,
        has_more,
    })
}
