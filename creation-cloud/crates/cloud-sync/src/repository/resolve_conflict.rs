//! 在设备与账号 revision 锁内实现全账号唯一、设备绑定的冲突解决幂等。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{ConflictResolution, ResolveConflictOutcome, ResolveConflictRequest, actor::SyncActor};

use super::{actor, change_set, checkpoint, state, storage};

type ResolutionRow = (Uuid, String, String, i64, Uuid, DateTime<Utc>);

pub(crate) async fn resolve_conflict(
    pool: &PgPool,
    sync_actor: &SyncActor,
    conflict_id: Uuid,
    request: &ResolveConflictRequest,
    fingerprint: &str,
) -> AppResult<ResolveConflictOutcome> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始冲突解决事务"))?;
    actor::lock_active(&mut transaction, sync_actor).await?;
    checkpoint::touch(&mut transaction, sync_actor).await?;
    let current_revision =
        state::lock_current_revision(&mut transaction, sync_actor.account_id()).await?;

    if let Some(stored) = find_resolution(&mut transaction, sync_actor, request).await? {
        let outcome = replay_outcome(sync_actor, conflict_id, request, fingerprint, stored)?;
        transaction
            .commit()
            .await
            .map_err(storage("无法结束冲突解决幂等事务"))?;
        return Ok(outcome);
    }

    lock_open_conflict(&mut transaction, sync_actor, conflict_id).await?;
    let revision = match request.changes() {
        Some(changes) => {
            change_set::apply(&mut transaction, sync_actor, current_revision, changes).await?
        }
        None => current_revision,
    };
    let resolved_at = save_resolution(
        &mut transaction,
        sync_actor,
        conflict_id,
        request,
        fingerprint,
        revision,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(storage("无法提交冲突解决事务"))?;

    Ok(ResolveConflictOutcome {
        conflict_id,
        resolution_mutation_id: request.resolution_mutation_id(),
        resolution: request.resolution(),
        revision,
        idempotent: false,
        resolved_at,
    })
}

async fn find_resolution(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    request: &ResolveConflictRequest,
) -> AppResult<Option<ResolutionRow>> {
    sqlx::query_as::<_, ResolutionRow>(
        "SELECT id, resolution, resolution_hash, resolved_revision, \
                resolved_by_device_id, resolved_at \
         FROM sync_conflicts \
         WHERE account_id = $1 AND resolution_mutation_id = $2",
    )
    .bind(actor.account_id())
    .bind(request.resolution_mutation_id())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage("无法读取冲突解决幂等记录"))
}

fn replay_outcome(
    actor: &SyncActor,
    conflict_id: Uuid,
    request: &ResolveConflictRequest,
    fingerprint: &str,
    stored: ResolutionRow,
) -> AppResult<ResolveConflictOutcome> {
    if stored.0 != conflict_id || stored.2 != fingerprint || stored.4 != actor.device_id() {
        return Err(AppError::Conflict(
            "resolution_mutation_id 已用于其它设备、冲突或不同内容".to_owned(),
        ));
    }
    let resolution = parse_resolution(&stored.1)?;
    if resolution != request.resolution() {
        return Err(AppError::Conflict(
            "resolution_mutation_id 已用于不同解决内容".to_owned(),
        ));
    }
    Ok(ResolveConflictOutcome {
        conflict_id: stored.0,
        resolution_mutation_id: request.resolution_mutation_id(),
        resolution,
        revision: stored.3,
        idempotent: true,
        resolved_at: stored.5,
    })
}

async fn lock_open_conflict(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    conflict_id: Uuid,
) -> AppResult<()> {
    let resolved = sqlx::query_scalar::<_, bool>(
        "SELECT resolved_at IS NOT NULL FROM sync_conflicts \
         WHERE account_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(actor.account_id())
    .bind(conflict_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage("无法锁定待解决同步冲突"))?
    .ok_or_else(|| AppError::NotFound("同步冲突不存在".to_owned()))?;
    if resolved {
        return Err(AppError::Conflict("同步冲突已经解决".to_owned()));
    }
    Ok(())
}

async fn save_resolution(
    transaction: &mut Transaction<'_, Postgres>,
    actor: &SyncActor,
    conflict_id: Uuid,
    request: &ResolveConflictRequest,
    fingerprint: &str,
    revision: i64,
) -> AppResult<DateTime<Utc>> {
    sqlx::query_scalar::<_, DateTime<Utc>>(
        "UPDATE sync_conflicts SET \
         resolved_at = now(), resolution = $3, resolution_mutation_id = $4, \
         resolution_hash = $5, resolved_revision = $6, resolved_by_device_id = $7 \
         WHERE account_id = $1 AND id = $2 AND resolved_at IS NULL \
         RETURNING resolved_at",
    )
    .bind(actor.account_id())
    .bind(conflict_id)
    .bind(request.resolution().as_str())
    .bind(request.resolution_mutation_id())
    .bind(fingerprint)
    .bind(revision)
    .bind(actor.device_id())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage("无法保存同步冲突解决结果"))
}

fn parse_resolution(value: &str) -> AppResult<ConflictResolution> {
    match value {
        "keep_remote" => Ok(ConflictResolution::KeepRemote),
        "apply_changes" => Ok(ConflictResolution::ApplyChanges),
        _ => Err(AppError::Internal("同步冲突解决类型无效".to_owned())),
    }
}
