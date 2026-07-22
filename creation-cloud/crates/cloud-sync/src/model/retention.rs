//! 定义同步保留任务的有界输入与分类删除计数。
//! 模型只校验批量边界，不执行数据库维护。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde::Serialize;

pub(crate) const MAX_RETENTION_BATCH_SIZE: u32 = 1_000;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct RetentionReport {
    pub tombstones_deleted: u64,
    pub applied_mutations_deleted: u64,
    pub resolved_conflicts_deleted: u64,
    pub conflict_mutations_deleted: u64,
}

impl RetentionReport {
    pub(crate) const fn is_empty(&self) -> bool {
        self.tombstones_deleted == 0
            && self.applied_mutations_deleted == 0
            && self.resolved_conflicts_deleted == 0
            && self.conflict_mutations_deleted == 0
    }

    pub(crate) fn absorb(&mut self, batch: Self) {
        self.tombstones_deleted += batch.tombstones_deleted;
        self.applied_mutations_deleted += batch.applied_mutations_deleted;
        self.resolved_conflicts_deleted += batch.resolved_conflicts_deleted;
        self.conflict_mutations_deleted += batch.conflict_mutations_deleted;
    }
}

pub(crate) struct RetentionRequest {
    retention_cutoff: DateTime<Utc>,
    active_cutoff: DateTime<Utc>,
    batch_size: i64,
}

impl RetentionRequest {
    pub(crate) fn new(
        retention_cutoff: DateTime<Utc>,
        active_cutoff: DateTime<Utc>,
        batch_size: u32,
    ) -> AppResult<Self> {
        if !(1..=MAX_RETENTION_BATCH_SIZE).contains(&batch_size) {
            return Err(AppError::Validation(format!(
                "batch_size 必须在 1 到 {MAX_RETENTION_BATCH_SIZE} 之间"
            )));
        }
        Ok(Self {
            retention_cutoff,
            active_cutoff,
            batch_size: i64::from(batch_size),
        })
    }

    pub(crate) const fn retention_cutoff(&self) -> DateTime<Utc> {
        self.retention_cutoff
    }

    pub(crate) const fn active_cutoff(&self) -> DateTime<Utc> {
        self.active_cutoff
    }

    pub(crate) const fn batch_size(&self) -> i64 {
        self.batch_size
    }
}
