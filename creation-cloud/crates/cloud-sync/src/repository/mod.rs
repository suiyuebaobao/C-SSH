//! 汇总同步领域按动作拆分的 PostgreSQL 仓储函数。

mod get_conflict;
mod list_conflicts;
mod pull;
mod push;
mod row;

pub(crate) use get_conflict::get_conflict;
pub(crate) use list_conflicts::list_conflicts;
pub(crate) use pull::pull;
pub(crate) use push::push;
pub(crate) use row::{ConflictRow, conflict_from_row};

use cloud_domain::AppError;

pub(crate) fn storage(message: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |_| AppError::Storage(message.to_owned())
}
