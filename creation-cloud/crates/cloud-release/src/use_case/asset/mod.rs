//! 按动作加载资产业务用例及对应独立测试。

pub(crate) mod create;
mod delete;
mod get;
pub(crate) mod installed_identity;
mod list;
mod list_all;
mod update;

#[cfg(test)]
mod tests;
