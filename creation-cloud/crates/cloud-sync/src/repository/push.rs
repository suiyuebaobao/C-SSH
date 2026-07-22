//! 在单个事务内实现设备绑定的 mutation 幂等、revision 竞争检查与冲突写入。

use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{PushOutcome, PushRequest, actor::SyncActor};

use super::{ConflictRow, actor, change_set, checkpoint, conflict_from_row, state, storage};

type MutationRow = (String, String, i64, Option<Uuid>, Option<Uuid>);

pub(crate) async fn push(
    pool: &PgPool,
    sync_actor: &SyncActor,
    request: &PushRequest,
    fingerprint: &str,
) -> AppResult<PushOutcome> {
    let mut transaction = pool.begin().await.map_err(storage("无法开始同步事务"))?;
    actor::lock_active(&mut transaction, sync_actor).await?;
    checkpoint::touch(&mut transaction, sync_actor).await?;
    let current_revision =
        state::lock_current_revision(&mut transaction, sync_actor.account_id()).await?;

    if let Some(stored) = find_mutation(&mut transaction, sync_actor, request).await? {
        let outcome = stored_outcome(&mut transaction, sync_actor, stored, fingerprint).await?;
        transaction
            .commit()
            .await
            .map_err(storage("无法结束同步幂等事务"))?;
        return Ok(outcome);
    }

    if request.base_revision != current_revision {
        return save_conflict(
            transaction,
            sync_actor,
            request,
            fingerprint,
            current_revision,
        )
        .await;
    }

    let revision = change_set::apply(
        &mut transaction,
        sync_actor,
        current_revision,
        &request.changes,
    )
    .await?;
    insert_mutation(
        &mut transaction,
        sync_actor,
        request,
        fingerprint,
        "applied",
        revision,
        None,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(storage("无法提交同步 mutation"))?;

    Ok(PushOutcome::Applied {
        revision,
        idempotent: false,
    })
}

async fn find_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: &PushRequest,
) -> AppResult<Option<MutationRow>> {
    sqlx::query_as::<_, MutationRow>(
        "SELECT outcome, mutation_hash, committed_revision, conflict_id, source_device_id \
         FROM sync_mutations WHERE account_id = $1 AND client_mutation_id = $2",
    )
    .bind(actor.account_id())
    .bind(request.client_mutation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage("无法读取同步幂等记录"))
}

async fn save_conflict(
    mut transaction: Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: &PushRequest,
    fingerprint: &str,
    current_revision: i64,
) -> AppResult<PushOutcome> {
    let conflict_id = Uuid::now_v7();
    let attempted = serde_json::to_value(&request.changes)
        .map_err(|_| AppError::Internal("无法编码同步冲突".to_owned()))?;
    let row = sqlx::query_as::<_, ConflictRow>(
        "INSERT INTO sync_conflicts \
         (id, account_id, client_mutation_id, base_revision, current_revision, \
          attempted_changes, source_device_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, client_mutation_id, base_revision, current_revision, \
                   attempted_changes, source_device_id, created_at",
    )
    .bind(conflict_id)
    .bind(actor.account_id())
    .bind(request.client_mutation_id)
    .bind(request.base_revision)
    .bind(current_revision)
    .bind(attempted)
    .bind(actor.device_id())
    .fetch_one(&mut *transaction)
    .await
    .map_err(storage("无法保存同步冲突"))?;

    insert_mutation(
        &mut transaction,
        actor,
        request,
        fingerprint,
        "conflict",
        current_revision,
        Some(conflict_id),
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(storage("无法提交同步冲突"))?;
    Ok(PushOutcome::Conflict {
        conflict: conflict_from_row(row)?,
        idempotent: false,
    })
}

async fn stored_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    stored: MutationRow,
    fingerprint: &str,
) -> AppResult<PushOutcome> {
    validate_replay_identity(actor, &stored.1, stored.4, fingerprint)?;
    if stored.0 == "applied" {
        return Ok(PushOutcome::Applied {
            revision: stored.2,
            idempotent: true,
        });
    }
    if stored.0 != "conflict" {
        return Err(AppError::Internal("同步幂等结果类型无效".to_owned()));
    }
    let conflict_id = stored
        .3
        .ok_or_else(|| AppError::Internal("同步冲突幂等记录缺少冲突标识".to_owned()))?;
    let row = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, client_mutation_id, base_revision, current_revision, attempted_changes, \
                source_device_id, created_at \
         FROM sync_conflicts WHERE account_id = $1 AND id = $2",
    )
    .bind(actor.account_id())
    .bind(conflict_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage("无法读取同步冲突幂等结果"))?;
    Ok(PushOutcome::Conflict {
        conflict: conflict_from_row(row)?,
        idempotent: true,
    })
}

pub(crate) fn validate_replay_identity(
    actor: &SyncActor,
    stored_fingerprint: &str,
    stored_device_id: Option<Uuid>,
    requested_fingerprint: &str,
) -> AppResult<()> {
    let same_device = stored_device_id.is_none_or(|device_id| device_id == actor.device_id());
    if stored_fingerprint == requested_fingerprint && same_device {
        return Ok(());
    }
    Err(AppError::Conflict(
        "client_mutation_id 已用于其它设备或不同内容".to_owned(),
    ))
}

async fn insert_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: &PushRequest,
    fingerprint: &str,
    outcome: &str,
    committed_revision: i64,
    conflict_id: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sync_mutations \
         (account_id, client_mutation_id, base_revision, mutation_hash, outcome, \
          committed_revision, conflict_id, source_device_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(actor.account_id())
    .bind(request.client_mutation_id)
    .bind(request.base_revision)
    .bind(fingerprint)
    .bind(outcome)
    .bind(committed_revision)
    .bind(conflict_id)
    .bind(actor.device_id())
    .execute(&mut **transaction)
    .await
    .map_err(storage("无法保存同步幂等记录"))?;
    Ok(())
}
