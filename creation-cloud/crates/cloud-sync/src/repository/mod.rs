//! 汇总同步领域按动作拆分的 PostgreSQL 仓储函数。

pub(crate) mod account_summary;
pub(crate) mod actor;
mod change_set;
pub(crate) mod checkpoint;
mod get_conflict;
mod list_conflicts;
pub(crate) mod pull;
mod push;
mod resolve_conflict;
pub(crate) mod retention;
mod row;
mod state;

pub(crate) use account_summary::account_summary;
pub(crate) use get_conflict::get_conflict;
pub(crate) use list_conflicts::list_conflicts;
pub(crate) use pull::pull;
pub(crate) use push::push;
#[cfg(test)]
pub(crate) use push::validate_replay_identity as validate_push_replay_identity;
pub(crate) use resolve_conflict::resolve_conflict;
pub(crate) use retention::run_batch as run_retention_batch;
pub(crate) use retention::run_batch_on_connection as run_retention_batch_on_connection;
pub(crate) use row::{ConflictRow, conflict_from_row};

use cloud_domain::AppError;

pub(crate) fn storage(message: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |_| AppError::Storage(message.to_owned())
}
