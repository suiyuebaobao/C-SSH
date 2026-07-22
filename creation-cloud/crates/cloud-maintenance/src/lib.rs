//! 提供低频维护任务的固定身份、独立锁、运行状态和只读备份观察能力。
//! 本包不查询任何业务表，也不执行会话、同步、下载或媒体领域动作。

mod backup;
mod connection_bounds;
mod lock;
mod model;
mod repository;
mod router;
mod service;
mod task;

pub use backup::{BackupCheck, BackupFileDeclaration, BackupManifest, check_latest_backup};
pub use lock::AdvisoryLock;
pub use model::{
    ErrorCode, MaintenanceStatus, ObservationCode, RunCompletion, RunOutcome, RunRecord, RunStart,
    RunTrigger, TaskExecutionReport,
};
pub use router::management_router;
pub use service::Service;
pub use task::MaintenanceTask;
