//! 数据保护幂等回执与重置挑战的有界保留查询。

use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::model::retention::RetentionRequest;

pub(crate) const DELETE_PROTECTION_MUTATIONS_SQL: &str = r#"
WITH candidates AS (
    SELECT mutation.account_id, mutation.mutation_id
    FROM cloud_data_protection_mutations AS mutation
    JOIN cloud_host_sync_states AS state ON state.account_id = mutation.account_id
    WHERE mutation.account_id = ANY($1::uuid[]) AND mutation.created_at < $2
      AND NOT (
          mutation.result_generation = state.sync_generation
          AND mutation.result_epoch = state.protection_epoch
          AND mutation.result_revision = state.protection_revision
          AND mutation.result_current_revision = state.current_revision
      )
    ORDER BY mutation.created_at, mutation.account_id, mutation.mutation_id
    FOR UPDATE OF mutation SKIP LOCKED
    LIMIT $3
)
DELETE FROM cloud_data_protection_mutations AS mutation
USING candidates
WHERE mutation.account_id = candidates.account_id
  AND mutation.mutation_id = candidates.mutation_id
RETURNING mutation.mutation_id
"#;

pub(crate) const DELETE_PROTECTION_RESET_CHALLENGES_SQL: &str = r#"
WITH candidates AS (
    SELECT challenge.account_id, challenge.id
    FROM cloud_data_protection_reset_challenges AS challenge
    WHERE challenge.account_id = ANY($1::uuid[])
      AND (
          challenge.consumed_at < $2
          OR (challenge.consumed_at IS NULL AND challenge.expires_at < $2)
      )
    ORDER BY COALESCE(challenge.consumed_at, challenge.expires_at),
             challenge.account_id, challenge.id
    FOR UPDATE OF challenge SKIP LOCKED
    LIMIT $3
)
DELETE FROM cloud_data_protection_reset_challenges AS challenge
USING candidates
WHERE challenge.account_id = candidates.account_id
  AND challenge.id = candidates.id
RETURNING challenge.id
"#;

pub(super) async fn delete(
    transaction: &mut Transaction<'_, Postgres>,
    account_ids: &[Uuid],
    request: &RetentionRequest,
) -> AppResult<(usize, usize)> {
    let mutations = super::delete_mutations(
        transaction,
        account_ids,
        request,
        DELETE_PROTECTION_MUTATIONS_SQL,
        "无法删除旧数据保护幂等记录",
    )
    .await?;
    let challenges = super::delete_mutations(
        transaction,
        account_ids,
        request,
        DELETE_PROTECTION_RESET_CHALLENGES_SQL,
        "无法删除旧数据保护重置挑战",
    )
    .await?;
    Ok((mutations, challenges))
}
