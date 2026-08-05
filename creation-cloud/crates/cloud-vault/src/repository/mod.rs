//! 汇总密文信封与包装密钥按 CRUD 动作拆分的 PostgreSQL 仓储函数。

mod create;
mod delete;
mod get;
mod list;
mod row;
mod update;
pub(crate) mod wrapper;

pub(crate) use create::create;
pub(crate) use delete::delete;
pub(crate) use get::get;
pub(crate) use list::list;
pub(crate) use row::{EnvelopeRow, envelope_from_row};
pub(crate) use update::update;

use cloud_domain::AppError;

pub(crate) fn storage(message: &'static str) -> impl FnOnce(sqlx::Error) -> AppError {
    move |_| AppError::Storage(message.to_owned())
}

pub(crate) fn delete_error(_: sqlx::Error) -> AppError {
    AppError::Storage("无法删除密文信封".to_owned())
}
