//! 汇总账号级包装密钥按 CRUD 动作拆分的 PostgreSQL 仓储函数。

pub(crate) mod create;
pub(crate) mod delete;
mod failure;
pub(crate) mod get;
pub(crate) mod list;
mod row;
pub(crate) mod update;

pub(crate) use row::{WrapperRow, wrapper_from_row};
