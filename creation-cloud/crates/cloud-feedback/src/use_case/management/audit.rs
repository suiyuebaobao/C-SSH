//! 统一收敛反馈语义审计事务，只有数据库提交成功后才通知通用审计中间件去重。

use cloud_domain::{AppError, AppResult, current_request_id, mark_semantic_audit_recorded};
use cloud_store::{Postgres, Transaction};
use uuid::Uuid;

use crate::repository;

pub(super) fn request_id() -> String {
    current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string())
}

pub(super) async fn commit_success<T>(
    transaction: Transaction<'_, Postgres>,
    value: T,
) -> AppResult<T> {
    commit(transaction).await?;
    Ok(value)
}

pub(super) async fn commit_failure<T>(
    transaction: Transaction<'_, Postgres>,
    error: AppError,
) -> AppResult<T> {
    commit(transaction).await?;
    Err(error)
}

pub(super) async fn rollback<T>(
    transaction: Transaction<'_, Postgres>,
    error: AppError,
) -> AppResult<T> {
    transaction
        .rollback()
        .await
        .map_err(repository::error::transaction)?;
    Err(error)
}

async fn commit(transaction: Transaction<'_, Postgres>) -> AppResult<()> {
    transaction
        .commit()
        .await
        .map_err(repository::error::transaction)?;
    mark_semantic_audit_recorded();
    Ok(())
}
