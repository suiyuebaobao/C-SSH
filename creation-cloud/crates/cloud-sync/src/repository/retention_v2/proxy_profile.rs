//! 清理已经安全确认的代理线路墓碑与旧密文版本。

use crate::{model::retention::RetentionRequest, repository::storage};
use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

const DELETE_TOMBSTONES: &str = r#"
WITH candidates AS (
    SELECT resource.account_id, resource.id, resource.revision
    FROM cloud_proxy_profiles AS resource
    WHERE resource.account_id = ANY($1::uuid[]) AND resource.is_deleted
      AND resource.updated_at < $2
      AND NOT EXISTS (
          SELECT 1 FROM cloud_sync_device_checkpoints AS checkpoint
          JOIN devices AS device ON device.account_id=checkpoint.account_id
                                AND device.id=checkpoint.device_id
          WHERE checkpoint.account_id=resource.account_id
            AND device.revoked_at IS NULL
            AND checkpoint.last_manual_sync_at >= $3
            AND checkpoint.acknowledged_revision < resource.revision
      )
    ORDER BY resource.updated_at,resource.account_id,resource.revision,resource.id
    FOR UPDATE OF resource SKIP LOCKED LIMIT $4
)
DELETE FROM cloud_proxy_profiles AS tombstone USING candidates
WHERE tombstone.account_id=candidates.account_id AND tombstone.id=candidates.id
RETURNING tombstone.account_id,candidates.revision
"#;

const DELETE_VERSIONS: &str = r#"
WITH safe_bounds AS MATERIALIZED (
    SELECT state.account_id,GREATEST(
        state.compacted_through_revision,
        COALESCE((
            SELECT MIN(checkpoint.acknowledged_revision)
            FROM cloud_sync_device_checkpoints AS checkpoint
            JOIN devices AS device ON device.account_id=checkpoint.account_id
                                  AND device.id=checkpoint.device_id
            WHERE checkpoint.account_id=state.account_id
              AND device.revoked_at IS NULL
              AND checkpoint.last_manual_sync_at >= $3
        ),state.current_revision)
    ) AS safe_revision
    FROM cloud_host_sync_states AS state WHERE state.account_id=ANY($1::uuid[])
), candidates AS MATERIALIZED (
    SELECT version.account_id,version.revision
    FROM cloud_proxy_profile_versions AS version
    JOIN safe_bounds AS safe ON safe.account_id=version.account_id
    WHERE version.recorded_at < $2 AND version.revision <= safe.safe_revision
      AND EXISTS (
          SELECT 1 FROM cloud_proxy_profile_versions AS newer
          WHERE newer.account_id=version.account_id
            AND newer.resource_id=version.resource_id
            AND newer.revision>version.revision
            AND newer.revision<=safe.safe_revision
      )
    ORDER BY version.recorded_at,version.account_id,version.revision
    FOR UPDATE OF version SKIP LOCKED LIMIT $4
), deleted AS (
    DELETE FROM cloud_proxy_profile_versions AS version USING candidates
    WHERE version.account_id=candidates.account_id AND version.revision=candidates.revision
    RETURNING version.account_id
)
SELECT deleted.account_id,safe.safe_revision FROM deleted
JOIN safe_bounds AS safe ON safe.account_id=deleted.account_id
"#;

pub(super) async fn delete(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
) -> AppResult<(Vec<(Uuid, i64)>, Vec<(Uuid, i64)>)> {
    let versions = sqlx::query_as(DELETE_VERSIONS)
        .bind(account_ids)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法删除代理线路同步版本历史"))?;
    let tombstones = sqlx::query_as(DELETE_TOMBSTONES)
        .bind(account_ids)
        .bind(request.retention_cutoff())
        .bind(request.active_cutoff())
        .bind(request.batch_size())
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage("无法删除代理线路同步墓碑"))?;
    Ok((versions, tombstones))
}
