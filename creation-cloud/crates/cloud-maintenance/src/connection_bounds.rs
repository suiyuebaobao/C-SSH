//! 统一维护 SQL 与终态事务的 PostgreSQL 会话等待上界。

pub(crate) const LOCK_WAIT_TIMEOUT_SECONDS: u64 = 3;
