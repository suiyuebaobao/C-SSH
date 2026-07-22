//! 在稳定账号锁序内执行墓碑、已应用 mutation 与已解决冲突的有界清理。
//! 墓碑删除和 compaction floor 推进位于同一事务，未解决冲突永不进入候选集。

use std::collections::BTreeMap;

use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::model::retention::{RetentionReport, RetentionRequest};

use super::storage;

pub(crate) const LOCK_CANDIDATE_ACCOUNTS_SQL: &str = r#"
SELECT account.id
FROM accounts AS account
WHERE EXISTS (
        SELECT 1 FROM sync_records AS record
        WHERE record.account_id = account.id
      AND record.is_deleted = TRUE
      AND record.updated_at < $1
      AND NOT EXISTS (
          SELECT 1
          FROM sync_device_checkpoints AS checkpoint
          JOIN devices AS device
            ON device.account_id = checkpoint.account_id
           AND device.id = checkpoint.device_id
          WHERE checkpoint.account_id = record.account_id
            AND device.revoked_at IS NULL
            AND checkpoint.last_sync_at >= $2
            AND checkpoint.acknowledged_revision < record.revision
      )
    )
   OR EXISTS (
        SELECT 1 FROM sync_mutations AS mutation
        WHERE mutation.account_id = account.id
          AND mutation.outcome = 'applied'
          AND mutation.created_at < $1
    )
   OR EXISTS (
        SELECT 1 FROM sync_conflicts AS conflict
        WHERE conflict.account_id = account.id
          AND conflict.resolved_at IS NOT NULL
          AND conflict.resolved_at < $1
    )
ORDER BY account.id ASC
FOR UPDATE SKIP LOCKED
LIMIT $3
"#;

pub(crate) const LOCK_DEVICES_SQL: &str = r#"
SELECT account_id, id
FROM devices
WHERE account_id = ANY($1::uuid[])
ORDER BY account_id ASC, id ASC
FOR SHARE
"#;

pub(crate) const DELETE_TOMBSTONES_SQL: &str = r#"
WITH candidates AS (
    SELECT record.id, record.account_id, record.revision
    FROM sync_records AS record
    WHERE record.account_id = ANY($1::uuid[])
      AND record.is_deleted = TRUE
      AND record.updated_at < $2
      AND NOT EXISTS (
          SELECT 1
          FROM sync_device_checkpoints AS checkpoint
          JOIN devices AS device
            ON device.account_id = checkpoint.account_id
           AND device.id = checkpoint.device_id
          WHERE checkpoint.account_id = record.account_id
            AND device.revoked_at IS NULL
            AND checkpoint.last_sync_at >= $3
            AND checkpoint.acknowledged_revision < record.revision
      )
    ORDER BY record.account_id ASC, record.updated_at ASC, record.revision ASC, record.id ASC
    FOR UPDATE OF record SKIP LOCKED
    LIMIT $4
)
DELETE FROM sync_records AS tombstone
USING candidates
WHERE tombstone.id = candidates.id
RETURNING tombstone.account_id, tombstone.revision
"#;

pub(crate) const ADVANCE_COMPACTION_FLOOR_SQL: &str = r#"
UPDATE sync_states
SET compacted_through_revision = LEAST(
    current_revision,
    GREATEST(compacted_through_revision, $2)
)
WHERE account_id = $1
"#;

pub(crate) const DELETE_APPLIED_MUTATIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT mutation.account_id, mutation.client_mutation_id
    FROM sync_mutations AS mutation
    WHERE mutation.account_id = ANY($1::uuid[])
      AND mutation.outcome = 'applied'
      AND mutation.created_at < $2
    ORDER BY mutation.created_at ASC, mutation.account_id ASC, mutation.client_mutation_id ASC
    FOR UPDATE OF mutation SKIP LOCKED
    LIMIT $3
)
DELETE FROM sync_mutations AS applied
USING candidates
WHERE applied.account_id = candidates.account_id
  AND applied.client_mutation_id = candidates.client_mutation_id
RETURNING applied.client_mutation_id
"#;

pub(crate) const DELETE_RESOLVED_CONFLICTS_SQL: &str = r#"
WITH candidates AS MATERIALIZED (
    SELECT conflict.account_id, conflict.id
    FROM sync_conflicts AS conflict
    WHERE conflict.account_id = ANY($1::uuid[])
      AND conflict.resolved_at IS NOT NULL
      AND conflict.resolved_at < $2
    ORDER BY conflict.resolved_at ASC, conflict.account_id ASC, conflict.id ASC
    FOR UPDATE OF conflict SKIP LOCKED
    LIMIT $3
),
linked_mutations AS MATERIALIZED (
    SELECT COUNT(*)::BIGINT AS count
    FROM sync_mutations AS mutation
    JOIN candidates
      ON candidates.account_id = mutation.account_id
     AND candidates.id = mutation.conflict_id
    WHERE mutation.outcome = 'conflict'
),
deleted AS (
    DELETE FROM sync_conflicts AS resolved
    USING candidates
    WHERE resolved.account_id = candidates.account_id
      AND resolved.id = candidates.id
    RETURNING resolved.id
)
SELECT (SELECT COUNT(*)::BIGINT FROM deleted),
       (SELECT count FROM linked_mutations)
"#;

const ENSURE_STATES_SQL: &str = r#"
INSERT INTO sync_states (account_id, current_revision)
SELECT account_id, 0
FROM unnest($1::uuid[]) AS candidate(account_id)
ON CONFLICT (account_id) DO NOTHING
"#;

const LOCK_STATES_SQL: &str = r#"
SELECT account_id
FROM sync_states
WHERE account_id = ANY($1::uuid[])
ORDER BY account_id ASC
FOR UPDATE
"#;

pub(crate) async fn run_batch(
    pool: &PgPool,
    request: &RetentionRequest,
) -> AppResult<RetentionReport> {
    let mut connection = pool
        .acquire()
        .await
        .map_err(storage("无法取得同步保留连接"))?;
    run_batch_on_connection(&mut connection, request).await
}

pub(crate) async fn run_batch_on_connection(
    connection: &mut PgConnection,
    request: &RetentionRequest,
) -> AppResult<RetentionReport> {
    let mut transaction = connection
        .begin()
        .await
        .map_err(storage("无法开始同步保留事务"))?;
    let account_ids = sqlx::query_scalar::<_, Uuid>(LOCK_CANDIDATE_ACCOUNTS_SQL)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法锁定同步保留账号"))?;
    if account_ids.is_empty() {
        transaction
            .commit()
            .await
            .map_err(storage("无法结束同步保留事务"))?;
        return Ok(RetentionReport::default());
    }

    // 维护任务沿用 account→device→state 顺序；不需要会话锁时不得倒序触碰状态行。
    sqlx::query_as::<_, (Uuid, Uuid)>(LOCK_DEVICES_SQL)
        .bind(account_ids.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法锁定同步保留设备"))?;
    sqlx::query(ENSURE_STATES_SQL)
        .bind(account_ids.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(storage("无法初始化同步保留状态"))?;
    sqlx::query_scalar::<_, Uuid>(LOCK_STATES_SQL)
        .bind(account_ids.as_slice())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法锁定同步保留状态"))?;

    let deleted_tombstones = sqlx::query_as::<_, (Uuid, i64)>(DELETE_TOMBSTONES_SQL)
        .bind(account_ids.as_slice())
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法删除同步墓碑"))?;
    advance_floors(&mut transaction, &deleted_tombstones).await?;

    let deleted_applied = sqlx::query_scalar::<_, Uuid>(DELETE_APPLIED_MUTATIONS_SQL)
        .bind(account_ids.as_slice())
        .bind(request.retention_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut *transaction)
        .await
        .map_err(storage("无法删除已应用同步幂等记录"))?;
    let deleted_resolved = sqlx::query_as::<_, (i64, i64)>(DELETE_RESOLVED_CONFLICTS_SQL)
        .bind(account_ids.as_slice())
        .bind(request.retention_cutoff())
        .bind(request.batch_size())
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage("无法删除已解决同步冲突"))?;

    transaction
        .commit()
        .await
        .map_err(storage("无法提交同步保留事务"))?;
    Ok(RetentionReport {
        tombstones_deleted: deleted_tombstones.len() as u64,
        applied_mutations_deleted: deleted_applied.len() as u64,
        resolved_conflicts_deleted: deleted_resolved.0 as u64,
        conflict_mutations_deleted: deleted_resolved.1 as u64,
    })
}

async fn advance_floors(
    transaction: &mut cloud_store::Transaction<'_, cloud_store::Postgres>,
    deleted_tombstones: &[(Uuid, i64)],
) -> AppResult<()> {
    let mut floors = BTreeMap::<Uuid, i64>::new();
    for (account_id, revision) in deleted_tombstones {
        floors
            .entry(*account_id)
            .and_modify(|current| *current = (*current).max(*revision))
            .or_insert(*revision);
    }
    for (account_id, revision) in floors {
        sqlx::query(ADVANCE_COMPACTION_FLOOR_SQL)
            .bind(account_id)
            .bind(revision)
            .execute(&mut **transaction)
            .await
            .map_err(storage("无法推进同步压缩边界"))?;
    }
    Ok(())
}
