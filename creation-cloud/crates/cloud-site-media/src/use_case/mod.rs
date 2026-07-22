//! 声明站点媒体各动作独立的业务用例，并把文件与数据库边界收敛在领域内部。

mod create;
mod delete;
mod get;
pub(crate) mod inspection;
mod list;
mod public;
mod publish;
mod revoke;
mod update;

#[cfg(test)]
mod tests;
