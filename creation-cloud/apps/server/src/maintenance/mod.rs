//! 装配五类维护任务的薄适配器、统一 runner 与可等待退出 supervisor。

mod adapters;
mod control;
mod progress;
mod runner;
mod supervisor;

pub(crate) use control::ShutdownSignal;
pub use runner::Runner;
pub use supervisor::Supervisor;
