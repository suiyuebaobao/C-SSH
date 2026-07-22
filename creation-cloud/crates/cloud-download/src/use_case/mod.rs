//! 汇总来源 CRUD 与公开分发用例。

mod account_history;
pub(crate) mod aggregation;
pub(crate) mod inspection;
mod public;
mod source;
mod upload;

#[cfg(test)]
mod upload_tests;
