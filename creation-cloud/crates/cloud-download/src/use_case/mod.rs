//! 汇总来源 CRUD 与公开分发用例。

mod account_history;
pub(crate) mod aggregation;
pub(crate) mod inspection;
mod installed_identity;
mod policy;
mod policy_apply;
#[cfg(test)]
mod policy_tests;
mod policy_validation;
mod public;
mod simplified_upload;
mod source;
mod upload;

#[cfg(test)]
mod upload_tests;
