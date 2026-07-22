//! 统一维护 SQL 与终态事务的 PostgreSQL 会话等待上界。

pub(crate) const LOCK_WAIT_TIMEOUT_SECONDS: u64 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_wait_bound_is_shorter_than_minimum_shutdown_window() {
        assert!(std::hint::black_box(LOCK_WAIT_TIMEOUT_SECONDS) < 5);
    }
}
