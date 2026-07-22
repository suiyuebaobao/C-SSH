//! 为业务执行与终态收敛建立彼此独立且有界的绝对截止时间。

use std::time::Duration;

use tokio::time::Instant;

const TERMINAL_SETTLE_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Copy)]
pub(super) struct RunBudget {
    pub(super) execution_deadline: Instant,
    pub(super) terminal_deadline: Instant,
}

impl RunBudget {
    pub(super) fn new(timeout: Duration) -> Self {
        let execution_deadline = Instant::now() + timeout;
        Self {
            execution_deadline,
            terminal_deadline: execution_deadline + TERMINAL_SETTLE_TIMEOUT,
        }
    }
}

#[cfg(test)]
#[path = "budget_tests.rs"]
mod tests;
