//! 汇总同步领域按动作拆分的 HTTP handler。

mod get_conflict;
mod list_conflicts;
mod pull;
mod push;

pub(crate) use get_conflict::get_conflict;
pub(crate) use list_conflicts::list_conflicts;
pub(crate) use pull::pull;
pub(crate) use push::push;
