//! 声明站点媒体各动作独立的 PostgreSQL repository。

pub(crate) mod create;
pub(crate) mod delete;
mod error;
pub(crate) mod finalization;
pub(crate) mod get;
pub(crate) mod inspection;
pub(crate) mod list;
pub(crate) mod public;
pub(crate) mod publish;
pub(crate) mod revoke;
pub(crate) mod update;

pub(crate) use error::{map_read_error, map_write_error};
