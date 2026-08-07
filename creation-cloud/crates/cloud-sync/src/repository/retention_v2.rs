//! 安全清理统一 Host/AI 同步的旧版本、墓碑与幂等 mutation 历史。
//! 所有删除都在账号锁和同步状态锁内完成，并同步推进不可越过的压缩边界。

use std::collections::BTreeMap;

use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::model::retention::{RetentionRequest, V2RetentionReport};

use super::storage;

pub(crate) const LOCK_CANDIDATE_ACCOUNTS_SQL: &str = r#"
SELECT account.id
FROM accounts AS account
WHERE EXISTS (
        SELECT 1 FROM cloud_hosts AS host
        WHERE host.account_id = account.id
          AND host.is_deleted
          AND host.updated_at < $1
          AND NOT EXISTS (
              SELECT 1
              FROM cloud_sync_device_checkpoints AS checkpoint
              JOIN devices AS device
                ON device.account_id = checkpoint.account_id
               AND device.id = checkpoint.device_id
              WHERE checkpoint.account_id = host.account_id
                AND device.revoked_at IS NULL
                AND checkpoint.last_manual_sync_at >= $2
                AND checkpoint.acknowledged_revision < host.revision
          )
    )
   OR EXISTS (
        SELECT 1 FROM cloud_ai_provider_configs AS resource
        WHERE resource.account_id = account.id
          AND resource.is_deleted
          AND resource.updated_at < $1
          AND NOT EXISTS (
              SELECT 1
              FROM cloud_sync_device_checkpoints AS checkpoint
              JOIN devices AS device
                ON device.account_id = checkpoint.account_id
               AND device.id = checkpoint.device_id
              WHERE checkpoint.account_id = resource.account_id
                AND device.revoked_at IS NULL
                AND checkpoint.last_manual_sync_at >= $2
                AND checkpoint.acknowledged_revision < resource.revision
          )
    )
   OR EXISTS (
        SELECT 1
        FROM cloud_host_versions AS version
        JOIN cloud_host_sync_states AS state ON state.account_id = version.account_id
        CROSS JOIN LATERAL (
            SELECT GREATEST(
                state.compacted_through_revision,
                COALESCE(
                    (
                        SELECT MIN(checkpoint.acknowledged_revision)
                        FROM cloud_sync_device_checkpoints AS checkpoint
                        JOIN devices AS device
                          ON device.account_id = checkpoint.account_id
                         AND device.id = checkpoint.device_id
                        WHERE checkpoint.account_id = state.account_id
                          AND device.revoked_at IS NULL
                          AND checkpoint.last_manual_sync_at >= $2
                    ),
                    state.current_revision
                )
            ) AS revision
        ) AS safe
        WHERE version.account_id = account.id
          AND version.recorded_at < $1
          AND version.revision <= safe.revision
          AND EXISTS (
              SELECT 1 FROM cloud_host_versions AS newer
              WHERE newer.account_id = version.account_id
                AND newer.host_id = version.host_id
                AND newer.revision > version.revision
                AND newer.revision <= safe.revision
          )
    )
   OR EXISTS (
        SELECT 1
        FROM cloud_ai_provider_config_versions AS version
        JOIN cloud_host_sync_states AS state ON state.account_id = version.account_id
        CROSS JOIN LATERAL (
            SELECT GREATEST(
                state.compacted_through_revision,
                COALESCE(
                    (
                        SELECT MIN(checkpoint.acknowledged_revision)
                        FROM cloud_sync_device_checkpoints AS checkpoint
                        JOIN devices AS device
                          ON device.account_id = checkpoint.account_id
                         AND device.id = checkpoint.device_id
                        WHERE checkpoint.account_id = state.account_id
                          AND device.revoked_at IS NULL
                          AND checkpoint.last_manual_sync_at >= $2
                    ),
                    state.current_revision
                )
            ) AS revision
        ) AS safe
        WHERE version.account_id = account.id
          AND version.recorded_at < $1
          AND version.revision <= safe.revision
          AND EXISTS (
              SELECT 1 FROM cloud_ai_provider_config_versions AS newer
              WHERE newer.account_id = version.account_id
                AND newer.resource_id = version.resource_id
                AND newer.revision > version.revision
                AND newer.revision <= safe.revision
          )
    )
   OR EXISTS (
        SELECT 1 FROM cloud_sync_push_mutations AS mutation
        WHERE mutation.account_id = account.id AND mutation.created_at < $1
    )
   OR EXISTS (
        SELECT 1 FROM cloud_sync_rekey_mutations AS mutation
        JOIN cloud_host_sync_states AS state ON state.account_id = mutation.account_id
        WHERE mutation.account_id = account.id AND mutation.created_at < $1
          AND mutation.result_generation <> state.sync_generation
    )
   OR EXISTS (
        SELECT 1 FROM cloud_sync_reset_mutations AS mutation
        JOIN cloud_host_sync_states AS state ON state.account_id = mutation.account_id
        WHERE mutation.account_id = account.id AND mutation.created_at < $1
          AND mutation.result_generation <> state.sync_generation
    )
ORDER BY account.id ASC
FOR UPDATE OF account SKIP LOCKED
LIMIT $3
"#;

pub(crate) const DELETE_HOST_TOMBSTONES_SQL: &str = r#"
WITH candidates AS (
    SELECT host.account_id, host.id, host.revision
    FROM cloud_hosts AS host
    WHERE host.account_id = ANY($1::uuid[])
      AND host.is_deleted
      AND host.updated_at < $2
      AND NOT EXISTS (
          SELECT 1
          FROM cloud_sync_device_checkpoints AS checkpoint
          JOIN devices AS device
            ON device.account_id = checkpoint.account_id
           AND device.id = checkpoint.device_id
          WHERE checkpoint.account_id = host.account_id
            AND device.revoked_at IS NULL
            AND checkpoint.last_manual_sync_at >= $3
            AND checkpoint.acknowledged_revision < host.revision
      )
    ORDER BY host.updated_at, host.account_id, host.revision, host.id
    FOR UPDATE OF host SKIP LOCKED
    LIMIT $4
)
DELETE FROM cloud_hosts AS tombstone
USING candidates
WHERE tombstone.account_id = candidates.account_id
  AND tombstone.id = candidates.id
RETURNING tombstone.account_id, candidates.revision
"#;

pub(crate) const DELETE_AI_TOMBSTONES_SQL: &str = r#"
WITH candidates AS (
    SELECT resource.account_id, resource.id, resource.revision
    FROM cloud_ai_provider_configs AS resource
    WHERE resource.account_id = ANY($1::uuid[])
      AND resource.is_deleted
      AND resource.updated_at < $2
      AND NOT EXISTS (
          SELECT 1
          FROM cloud_sync_device_checkpoints AS checkpoint
          JOIN devices AS device
            ON device.account_id = checkpoint.account_id
           AND device.id = checkpoint.device_id
          WHERE checkpoint.account_id = resource.account_id
            AND device.revoked_at IS NULL
            AND checkpoint.last_manual_sync_at >= $3
            AND checkpoint.acknowledged_revision < resource.revision
      )
    ORDER BY resource.updated_at, resource.account_id, resource.revision, resource.id
    FOR UPDATE OF resource SKIP LOCKED
    LIMIT $4
)
DELETE FROM cloud_ai_provider_configs AS tombstone
USING candidates
WHERE tombstone.account_id = candidates.account_id
  AND tombstone.id = candidates.id
RETURNING tombstone.account_id, candidates.revision
"#;

pub(crate) const DELETE_HOST_VERSIONS_SQL: &str = r#"
WITH safe_bounds AS MATERIALIZED (
    SELECT state.account_id,
           GREATEST(
               state.compacted_through_revision,
               COALESCE(
                   (
                       SELECT MIN(checkpoint.acknowledged_revision)
                       FROM cloud_sync_device_checkpoints AS checkpoint
                       JOIN devices AS device
                         ON device.account_id = checkpoint.account_id
                        AND device.id = checkpoint.device_id
                       WHERE checkpoint.account_id = state.account_id
                         AND device.revoked_at IS NULL
                         AND checkpoint.last_manual_sync_at >= $3
                   ),
                   state.current_revision
               )
           ) AS safe_revision
    FROM cloud_host_sync_states AS state
    WHERE state.account_id = ANY($1::uuid[])
), candidates AS MATERIALIZED (
    SELECT version.account_id, version.revision
    FROM cloud_host_versions AS version
    JOIN safe_bounds AS safe ON safe.account_id = version.account_id
    WHERE version.recorded_at < $2
      AND version.revision <= safe.safe_revision
      AND EXISTS (
          SELECT 1 FROM cloud_host_versions AS newer
          WHERE newer.account_id = version.account_id
            AND newer.host_id = version.host_id
            AND newer.revision > version.revision
            AND newer.revision <= safe.safe_revision
      )
    ORDER BY version.recorded_at, version.account_id, version.revision
    FOR UPDATE OF version SKIP LOCKED
    LIMIT $4
), deleted AS (
    DELETE FROM cloud_host_versions AS version
    USING candidates
    WHERE version.account_id = candidates.account_id
      AND version.revision = candidates.revision
    RETURNING version.account_id
)
SELECT deleted.account_id, safe.safe_revision
FROM deleted
JOIN safe_bounds AS safe ON safe.account_id = deleted.account_id
"#;

pub(crate) const DELETE_AI_VERSIONS_SQL: &str = r#"
WITH safe_bounds AS MATERIALIZED (
    SELECT state.account_id,
           GREATEST(
               state.compacted_through_revision,
               COALESCE(
                   (
                       SELECT MIN(checkpoint.acknowledged_revision)
                       FROM cloud_sync_device_checkpoints AS checkpoint
                       JOIN devices AS device
                         ON device.account_id = checkpoint.account_id
                        AND device.id = checkpoint.device_id
                       WHERE checkpoint.account_id = state.account_id
                         AND device.revoked_at IS NULL
                         AND checkpoint.last_manual_sync_at >= $3
                   ),
                   state.current_revision
               )
           ) AS safe_revision
    FROM cloud_host_sync_states AS state
    WHERE state.account_id = ANY($1::uuid[])
), candidates AS MATERIALIZED (
    SELECT version.account_id, version.revision
    FROM cloud_ai_provider_config_versions AS version
    JOIN safe_bounds AS safe ON safe.account_id = version.account_id
    WHERE version.recorded_at < $2
      AND version.revision <= safe.safe_revision
      AND EXISTS (
          SELECT 1 FROM cloud_ai_provider_config_versions AS newer
          WHERE newer.account_id = version.account_id
            AND newer.resource_id = version.resource_id
            AND newer.revision > version.revision
            AND newer.revision <= safe.safe_revision
      )
    ORDER BY version.recorded_at, version.account_id, version.revision
    FOR UPDATE OF version SKIP LOCKED
    LIMIT $4
), deleted AS (
    DELETE FROM cloud_ai_provider_config_versions AS version
    USING candidates
    WHERE version.account_id = candidates.account_id
      AND version.revision = candidates.revision
    RETURNING version.account_id
)
SELECT deleted.account_id, safe.safe_revision
FROM deleted
JOIN safe_bounds AS safe ON safe.account_id = deleted.account_id
"#;

pub(crate) const DELETE_PUSH_MUTATIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT mutation.account_id, mutation.client_mutation_id
    FROM cloud_sync_push_mutations AS mutation
    WHERE mutation.account_id = ANY($1::uuid[]) AND mutation.created_at < $2
    ORDER BY mutation.created_at, mutation.account_id, mutation.client_mutation_id
    FOR UPDATE OF mutation SKIP LOCKED
    LIMIT $3
)
DELETE FROM cloud_sync_push_mutations AS mutation
USING candidates
WHERE mutation.account_id = candidates.account_id
  AND mutation.client_mutation_id = candidates.client_mutation_id
RETURNING mutation.client_mutation_id
"#;

pub(crate) const DELETE_REKEY_MUTATIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT mutation.account_id, mutation.mutation_id
    FROM cloud_sync_rekey_mutations AS mutation
    JOIN cloud_host_sync_states AS state ON state.account_id = mutation.account_id
    WHERE mutation.account_id = ANY($1::uuid[]) AND mutation.created_at < $2
      AND mutation.result_generation <> state.sync_generation
    ORDER BY mutation.created_at, mutation.account_id, mutation.mutation_id
    FOR UPDATE OF mutation SKIP LOCKED
    LIMIT $3
)
DELETE FROM cloud_sync_rekey_mutations AS mutation
USING candidates
WHERE mutation.account_id = candidates.account_id
  AND mutation.mutation_id = candidates.mutation_id
RETURNING mutation.mutation_id
"#;

pub(crate) const DELETE_RESET_MUTATIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT mutation.account_id, mutation.mutation_id
    FROM cloud_sync_reset_mutations AS mutation
    JOIN cloud_host_sync_states AS state ON state.account_id = mutation.account_id
    WHERE mutation.account_id = ANY($1::uuid[]) AND mutation.created_at < $2
      AND mutation.result_generation <> state.sync_generation
    ORDER BY mutation.created_at, mutation.account_id, mutation.mutation_id
    FOR UPDATE OF mutation SKIP LOCKED
    LIMIT $3
)
DELETE FROM cloud_sync_reset_mutations AS mutation
USING candidates
WHERE mutation.account_id = candidates.account_id
  AND mutation.mutation_id = candidates.mutation_id
RETURNING mutation.mutation_id
"#;

const ENSURE_STATES_SQL: &str = r#"
INSERT INTO cloud_host_sync_states (account_id)
SELECT account_id FROM unnest($1::uuid[]) AS candidate(account_id)
ON CONFLICT (account_id) DO NOTHING
"#;

const LOCK_STATES_SQL: &str = r#"
SELECT account_id
FROM cloud_host_sync_states
WHERE account_id = ANY($1::uuid[])
ORDER BY account_id
FOR UPDATE
"#;

const ADVANCE_FLOOR_SQL: &str = r#"
UPDATE cloud_host_sync_states
SET compacted_through_revision = LEAST(
    current_revision,
    GREATEST(compacted_through_revision, $2)
)
WHERE account_id = $1
"#;

pub(crate) async fn lock_candidate_accounts(
    transaction: &mut Transaction<'_, Postgres>,
    request: &RetentionRequest,
) -> AppResult<Vec<Uuid>> {
    sqlx::query_scalar(LOCK_CANDIDATE_ACCOUNTS_SQL)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法锁定统一密文同步保留账号"))
}

pub(crate) async fn run(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
) -> AppResult<V2RetentionReport> {
    sqlx::query(ENSURE_STATES_SQL)
        .bind(account_ids)
        .execute(&mut **transaction)
        .await
        .map_err(storage("无法初始化统一密文同步状态"))?;
    sqlx::query_scalar::<_, Uuid>(LOCK_STATES_SQL)
        .bind(account_ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法锁定统一密文同步状态"))?;

    let host_versions = delete_versions(transaction, account_ids, request, true).await?;
    let ai_versions = delete_versions(transaction, account_ids, request, false).await?;
    let host_tombstones = delete_tombstones(transaction, account_ids, request, true).await?;
    let ai_tombstones = delete_tombstones(transaction, account_ids, request, false).await?;
    advance_floors(
        transaction,
        host_versions
            .iter()
            .chain(&ai_versions)
            .chain(&host_tombstones)
            .chain(&ai_tombstones),
    )
    .await?;

    let push_mutations = delete_mutations(
        transaction,
        account_ids,
        request,
        DELETE_PUSH_MUTATIONS_SQL,
        "无法删除旧 push 幂等记录",
    )
    .await?;
    let rekey_mutations = delete_mutations(
        transaction,
        account_ids,
        request,
        DELETE_REKEY_MUTATIONS_SQL,
        "无法删除旧 rekey 幂等记录",
    )
    .await?;
    let reset_mutations = delete_mutations(
        transaction,
        account_ids,
        request,
        DELETE_RESET_MUTATIONS_SQL,
        "无法删除旧 reset 幂等记录",
    )
    .await?;

    Ok(V2RetentionReport {
        tombstones_deleted: (host_tombstones.len() + ai_tombstones.len()) as u64,
        versions_deleted: (host_versions.len() + ai_versions.len()) as u64,
        mutations_deleted: (push_mutations + rekey_mutations + reset_mutations) as u64,
    })
}

async fn delete_tombstones(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
    hosts: bool,
) -> AppResult<Vec<(Uuid, i64)>> {
    let sql = if hosts {
        DELETE_HOST_TOMBSTONES_SQL
    } else {
        DELETE_AI_TOMBSTONES_SQL
    };
    sqlx::query_as(sql)
        .bind(account_ids)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法删除统一密文同步墓碑"))
}

async fn delete_versions(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
    hosts: bool,
) -> AppResult<Vec<(Uuid, i64)>> {
    let sql = if hosts {
        DELETE_HOST_VERSIONS_SQL
    } else {
        DELETE_AI_VERSIONS_SQL
    };
    sqlx::query_as(sql)
        .bind(account_ids)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法删除统一密文同步版本历史"))
}

async fn delete_mutations(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
    sql: &str,
    error: &'static str,
) -> AppResult<usize> {
    sqlx::query_scalar::<_, Uuid>(sql)
        .bind(account_ids)
        .bind(request.retention_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map(|rows| rows.len())
        .map_err(storage(error))
}

async fn advance_floors<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    rows: impl Iterator<Item = &'a (Uuid, i64)>,
) -> AppResult<()> {
    let mut floors = BTreeMap::<Uuid, i64>::new();
    for (account_id, revision) in rows {
        floors
            .entry(*account_id)
            .and_modify(|current| *current = (*current).max(*revision))
            .or_insert(*revision);
    }
    for (account_id, revision) in floors {
        sqlx::query(ADVANCE_FLOOR_SQL)
            .bind(account_id)
            .bind(revision)
            .execute(&mut **transaction)
            .await
            .map_err(storage("无法推进统一密文同步压缩边界"))?;
    }
    Ok(())
}
