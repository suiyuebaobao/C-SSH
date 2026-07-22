//! 初始化并锁定账号唯一同步修订行，供 push 与冲突解决共享串行边界。

use cloud_domain::AppResult;
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use super::storage;

pub(crate) const SHARE_REVISION_BOUNDS_SQL: &str = r#"
SELECT current_revision, compacted_through_revision
FROM sync_states
WHERE account_id = $1
FOR SHARE
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RevisionBounds {
    pub(crate) current_revision: i64,
    pub(crate) compacted_through_revision: i64,
}

pub(crate) async fn lock_current_revision(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<i64> {
    ensure_state(transaction, account_id).await?;

    sqlx::query_scalar::<_, i64>(
        "SELECT current_revision FROM sync_states WHERE account_id = $1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage("无法锁定同步修订号"))
}

pub(crate) async fn share_revision_bounds(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<RevisionBounds> {
    ensure_state(transaction, account_id).await?;
    sqlx::query_as::<_, (i64, i64)>(SHARE_REVISION_BOUNDS_SQL)
        .bind(account_id)
        .fetch_one(&mut **transaction)
        .await
        .map(|row| RevisionBounds {
            current_revision: row.0,
            compacted_through_revision: row.1,
        })
        .map_err(storage("无法读取同步修订边界"))
}

async fn ensure_state(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO sync_states (account_id, current_revision) VALUES ($1, 0) \
         ON CONFLICT (account_id) DO NOTHING",
    )
    .bind(account_id)
    .execute(&mut **transaction)
    .await
    .map_err(storage("无法初始化同步状态"))?;
    Ok(())
}
