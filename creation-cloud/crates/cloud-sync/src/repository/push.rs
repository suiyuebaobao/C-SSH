//! 在单个事务内实现 mutation 幂等、revision 竞争检查、冲突和墓碑写入。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{PushOutcome, PushRequest, SyncOperation};

use super::{ConflictRow, conflict_from_row, storage};

type MutationRow = (String, String, i64, Option<Uuid>);

pub(crate) async fn push(
    pool: &PgPool,
    account_id: Uuid,
    request: &PushRequest,
    fingerprint: &str,
) -> AppResult<PushOutcome> {
    let mut transaction = pool.begin().await.map_err(storage("无法开始同步事务"))?;

    sqlx::query(
        "INSERT INTO sync_states (account_id, current_revision) VALUES ($1, 0) \
         ON CONFLICT (account_id) DO NOTHING",
    )
    .bind(account_id)
    .execute(&mut *transaction)
    .await
    .map_err(storage("无法初始化同步状态"))?;

    // 同一账号的 push 先串行锁定 revision，确保并发重放能看到已提交的幂等记录。
    let current_revision = sqlx::query_scalar::<_, i64>(
        "SELECT current_revision FROM sync_states WHERE account_id = $1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(storage("无法锁定同步修订号"))?;

    if let Some(stored) = sqlx::query_as::<_, MutationRow>(
        "SELECT outcome, mutation_hash, committed_revision, conflict_id \
         FROM sync_mutations WHERE account_id = $1 AND client_mutation_id = $2",
    )
    .bind(account_id)
    .bind(request.client_mutation_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage("无法读取同步幂等记录"))?
    {
        let outcome = stored_outcome(&mut transaction, account_id, stored, fingerprint).await?;
        transaction
            .commit()
            .await
            .map_err(storage("无法结束同步幂等事务"))?;
        return Ok(outcome);
    }

    if request.base_revision != current_revision {
        let conflict_id = Uuid::now_v7();
        let attempted = serde_json::to_value(&request.changes)
            .map_err(|_| AppError::Internal("无法编码同步冲突".to_owned()))?;
        let row = sqlx::query_as::<_, ConflictRow>(
            "INSERT INTO sync_conflicts \
             (id, account_id, client_mutation_id, base_revision, current_revision, attempted_changes) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, client_mutation_id, base_revision, current_revision, attempted_changes, created_at",
        )
        .bind(conflict_id)
        .bind(account_id)
        .bind(request.client_mutation_id)
        .bind(request.base_revision)
        .bind(current_revision)
        .bind(attempted)
        .fetch_one(&mut *transaction)
        .await
        .map_err(storage("无法保存同步冲突"))?;

        insert_mutation(
            &mut transaction,
            account_id,
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
        return Ok(PushOutcome::Conflict {
            conflict: conflict_from_row(row)?,
            idempotent: false,
        });
    }

    let mut revision = current_revision;
    for change in &request.changes {
        revision += 1;
        match change.operation {
            SyncOperation::Upsert => {
                sqlx::query(
                    "INSERT INTO sync_records \
                     (id, account_id, namespace, record_key, revision, value, is_deleted) \
                     VALUES ($1, $2, $3, $4, $5, $6, FALSE) \
                     ON CONFLICT (account_id, namespace, record_key) DO UPDATE SET \
                     revision = EXCLUDED.revision, value = EXCLUDED.value, \
                     is_deleted = FALSE, updated_at = now()",
                )
                .bind(Uuid::now_v7())
                .bind(account_id)
                .bind(&change.namespace)
                .bind(&change.key)
                .bind(revision)
                .bind(change.value.as_ref())
                .execute(&mut *transaction)
                .await
                .map_err(storage("无法写入同步记录"))?;
            }
            SyncOperation::Delete => {
                sqlx::query(
                    "INSERT INTO sync_records \
                     (id, account_id, namespace, record_key, revision, value, is_deleted) \
                     VALUES ($1, $2, $3, $4, $5, NULL, TRUE) \
                     ON CONFLICT (account_id, namespace, record_key) DO UPDATE SET \
                     revision = EXCLUDED.revision, value = NULL, \
                     is_deleted = TRUE, updated_at = now()",
                )
                .bind(Uuid::now_v7())
                .bind(account_id)
                .bind(&change.namespace)
                .bind(&change.key)
                .bind(revision)
                .execute(&mut *transaction)
                .await
                .map_err(storage("无法写入同步墓碑"))?;
            }
        }
    }

    sqlx::query("UPDATE sync_states SET current_revision = $2 WHERE account_id = $1")
        .bind(account_id)
        .bind(revision)
        .execute(&mut *transaction)
        .await
        .map_err(storage("无法推进同步修订号"))?;
    insert_mutation(
        &mut transaction,
        account_id,
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

async fn stored_outcome(
    transaction: &mut cloud_store::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
    stored: MutationRow,
    fingerprint: &str,
) -> AppResult<PushOutcome> {
    if stored.1 != fingerprint {
        return Err(AppError::Conflict(
            "client_mutation_id 已用于不同内容".to_owned(),
        ));
    }
    if stored.0 == "applied" {
        return Ok(PushOutcome::Applied {
            revision: stored.2,
            idempotent: true,
        });
    }
    let conflict_id = stored
        .3
        .ok_or_else(|| AppError::Internal("同步冲突幂等记录缺少冲突标识".to_owned()))?;
    let row = sqlx::query_as::<_, ConflictRow>(
        "SELECT id, client_mutation_id, base_revision, current_revision, attempted_changes, created_at \
         FROM sync_conflicts WHERE account_id = $1 AND id = $2",
    )
    .bind(account_id)
    .bind(conflict_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage("无法读取同步冲突幂等结果"))?;
    Ok(PushOutcome::Conflict {
        conflict: conflict_from_row(row)?,
        idempotent: true,
    })
}

async fn insert_mutation(
    transaction: &mut cloud_store::Transaction<'_, sqlx::Postgres>,
    account_id: Uuid,
    request: &PushRequest,
    fingerprint: &str,
    outcome: &str,
    committed_revision: i64,
    conflict_id: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sync_mutations \
         (account_id, client_mutation_id, base_revision, mutation_hash, outcome, \
          committed_revision, conflict_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(account_id)
    .bind(request.client_mutation_id)
    .bind(request.base_revision)
    .bind(fingerprint)
    .bind(outcome)
    .bind(committed_revision)
    .bind(conflict_id)
    .execute(&mut **transaction)
    .await
    .map_err(storage("无法保存同步幂等记录"))?;
    Ok(())
}
