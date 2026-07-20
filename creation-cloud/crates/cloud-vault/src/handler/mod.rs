//! 汇总密文信封按 CRUD 动作拆分的 HTTP handler。

mod create;
mod delete;
mod get;
mod list;
mod update;

pub(crate) use create::create;
pub(crate) use delete::delete;
pub(crate) use get::get;
pub(crate) use list::list;
pub(crate) use update::update;
