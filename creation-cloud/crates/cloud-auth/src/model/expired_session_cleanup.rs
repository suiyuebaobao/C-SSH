//! 定义过期会话清理的截止时间与有界批量参数。
//! 本模型只负责拒绝无界维护请求，不接触数据库。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};

pub(crate) const MAX_EXPIRED_SESSION_BATCH_SIZE: u32 = 1_000;

pub(crate) struct ExpiredSessionCleanup {
    delete_before: DateTime<Utc>,
    batch_size: i64,
}

impl ExpiredSessionCleanup {
    pub(crate) fn new(delete_before: DateTime<Utc>, batch_size: u32) -> AppResult<Self> {
        if !(1..=MAX_EXPIRED_SESSION_BATCH_SIZE).contains(&batch_size) {
            return Err(AppError::Validation(format!(
                "batch_size 必须在 1 到 {MAX_EXPIRED_SESSION_BATCH_SIZE} 之间"
            )));
        }
        Ok(Self {
            delete_before,
            batch_size: i64::from(batch_size),
        })
    }

    pub(crate) const fn delete_before(&self) -> DateTime<Utc> {
        self.delete_before
    }

    pub(crate) const fn batch_size(&self) -> i64 {
        self.batch_size
    }
}
