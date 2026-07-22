//! 在账号修订共享锁内执行增量拉取或全量重建分页。
//! 增量游标通过当前修订证明后才可确认；全量模式只刷新设备活动。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use serde_json::Value;

use crate::{PullMode, PullRequest, PullResponse, SyncRecord, actor::SyncActor};

use super::{actor, checkpoint, state, storage};

pub(crate) const INCREMENTAL_PULL_SQL: &str = r#"
SELECT namespace, record_key, revision, value, is_deleted, source_device_id, updated_at
FROM sync_records
WHERE account_id = $1 AND revision > $2
ORDER BY revision ASC, namespace ASC, record_key ASC
LIMIT $3
"#;

pub(crate) const FULL_PULL_SQL: &str = r#"
WITH snapshot_records AS MATERIALIZED (
    SELECT DISTINCT ON (namespace, record_key)
           namespace, record_key, revision, value, is_deleted, source_device_id, recorded_at
    FROM sync_record_versions
    WHERE account_id = $1 AND revision <= $3
    ORDER BY namespace ASC, record_key ASC, revision DESC
)
SELECT namespace, record_key, revision, value, is_deleted, source_device_id, recorded_at
FROM snapshot_records
WHERE revision > $2
  AND is_deleted = FALSE
ORDER BY revision ASC, namespace ASC, record_key ASC
LIMIT $4
"#;

type RecordRow = (
    String,
    String,
    i64,
    Option<Value>,
    bool,
    Option<uuid::Uuid>,
    DateTime<Utc>,
);

pub(crate) async fn pull(
    pool: &PgPool,
    sync_actor: &SyncActor,
    request: PullRequest,
) -> AppResult<PullResponse> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始同步拉取事务"))?;
    actor::share_active(&mut transaction, sync_actor).await?;
    let bounds = state::share_revision_bounds(&mut transaction, sync_actor.account_id()).await?;

    match request.mode {
        PullMode::Incremental => incremental(transaction, sync_actor, request, bounds).await,
        PullMode::Full => full(transaction, sync_actor, request, bounds).await,
    }
}

async fn incremental(
    mut transaction: Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: PullRequest,
    bounds: state::RevisionBounds,
) -> AppResult<PullResponse> {
    if request.since_revision > bounds.current_revision {
        checkpoint::touch(&mut transaction, actor).await?;
        commit(transaction).await?;
        return Err(AppError::Validation(
            "since_revision 不能超过当前同步修订".to_owned(),
        ));
    }
    if request.since_revision < bounds.compacted_through_revision {
        checkpoint::touch(&mut transaction, actor).await?;
        commit(transaction).await?;
        return Err(AppError::SyncResyncRequired(
            "增量游标早于服务端保留边界，请执行全量重建".to_owned(),
        ));
    }

    // 只有客户端提交且已证明不超过当前修订的增量游标，才可推进确认点。
    checkpoint::acknowledge_incremental(&mut transaction, actor, request.since_revision).await?;
    let fetch_limit = fetch_limit(request.limit)?;
    let rows = sqlx::query_as::<_, RecordRow>(INCREMENTAL_PULL_SQL)
        .bind(actor.account_id())
        .bind(request.since_revision)
        .bind(fetch_limit)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法拉取增量同步记录"))?;
    let (records, has_more) = complete_revision_page(map_rows(rows), request.limit);
    let next_revision = records
        .last()
        .map_or(request.since_revision, |record| record.revision);
    commit(transaction).await?;
    Ok(PullResponse {
        records,
        next_revision,
        has_more,
        snapshot_revision: None,
    })
}

async fn full(
    mut transaction: Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: PullRequest,
    bounds: state::RevisionBounds,
) -> AppResult<PullResponse> {
    checkpoint::touch(&mut transaction, actor).await?;
    let snapshot_revision = request.snapshot_revision.unwrap_or(bounds.current_revision);
    if snapshot_revision > bounds.current_revision {
        commit(transaction).await?;
        return Err(AppError::Validation(
            "snapshot_revision 不能超过当前同步修订".to_owned(),
        ));
    }
    if snapshot_revision < bounds.compacted_through_revision {
        commit(transaction).await?;
        return Err(AppError::SyncResyncRequired(
            "全量快照早于服务端保留边界，请重新开始全量重建".to_owned(),
        ));
    }

    let fetch_limit = fetch_limit(request.limit)?;
    let rows = sqlx::query_as::<_, RecordRow>(FULL_PULL_SQL)
        .bind(actor.account_id())
        .bind(request.since_revision)
        .bind(snapshot_revision)
        .bind(fetch_limit)
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法拉取全量同步记录"))?;
    let (records, has_more) = complete_revision_page(map_rows(rows), request.limit);
    let next_revision = if has_more {
        records
            .last()
            .map_or(request.since_revision, |record| record.revision)
    } else {
        snapshot_revision
    };
    commit(transaction).await?;
    Ok(PullResponse {
        records,
        next_revision,
        has_more,
        snapshot_revision: Some(snapshot_revision),
    })
}

fn map_rows(rows: Vec<RecordRow>) -> Vec<SyncRecord> {
    rows.into_iter()
        .map(|row| SyncRecord {
            namespace: row.0,
            key: row.1,
            revision: row.2,
            value: row.3,
            deleted: row.4,
            source_device_id: row.5,
            updated_at: row.6,
        })
        .collect()
}

pub(crate) fn complete_revision_page(
    mut records: Vec<SyncRecord>,
    limit: u32,
) -> (Vec<SyncRecord>, bool) {
    let limit = limit as usize;
    if records.len() <= limit {
        return (records, false);
    }
    let boundary_revision = records[limit - 1].revision;
    let next_revision_start = records[limit..]
        .iter()
        .position(|record| record.revision != boundary_revision)
        .map(|offset| limit + offset);
    match next_revision_start {
        Some(index) => {
            records.truncate(index);
            (records, true)
        }
        None => (records, false),
    }
}

fn fetch_limit(limit: u32) -> AppResult<i64> {
    let maximum_revision_group = i64::try_from(crate::validation::MAX_CHANGES)
        .map_err(|_| AppError::Internal("同步 revision 分组上限无效".to_owned()))?;
    Ok(i64::from(limit) + maximum_revision_group + 1)
}

async fn commit(transaction: Transaction<'_, Postgres>) -> AppResult<()> {
    transaction
        .commit()
        .await
        .map_err(storage("无法结束同步拉取事务"))
}
