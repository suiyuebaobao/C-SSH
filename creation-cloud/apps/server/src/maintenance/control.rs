//! 在维护 runner、CLI 与 supervisor 之间传递同一个绝对关停截止时间。

use std::future::Future;

use tokio::{sync::watch, time::Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShutdownSignal {
    Running,
    Requested(Instant),
}

impl ShutdownSignal {
    #[must_use]
    pub(crate) const fn deadline(self) -> Option<Instant> {
        match self {
            Self::Running => None,
            Self::Requested(deadline) => Some(deadline),
        }
    }
}

pub(super) enum PhaseResult<T> {
    Completed(T),
    Cancelled(Instant),
    TimedOut,
}

pub(super) async fn run_until<F>(
    shutdown: &mut watch::Receiver<ShutdownSignal>,
    deadline: Instant,
    operation: F,
) -> PhaseResult<F::Output>
where
    F: Future,
{
    if let Some(deadline) = shutdown.borrow().deadline() {
        return PhaseResult::Cancelled(deadline);
    }
    if deadline <= Instant::now() {
        return PhaseResult::TimedOut;
    }
    tokio::select! {
        biased;
        requested = wait_for_request(shutdown) => PhaseResult::Cancelled(requested),
        () = tokio::time::sleep_until(deadline) => PhaseResult::TimedOut,
        output = operation => PhaseResult::Completed(output),
    }
}

pub(super) async fn wait_for_request(shutdown: &mut watch::Receiver<ShutdownSignal>) -> Instant {
    loop {
        if let Some(deadline) = shutdown.borrow().deadline() {
            return deadline;
        }
        if shutdown.changed().await.is_err() {
            // 控制端异常消失时失败关闭，立即停止可能产生新写入的阶段。
            return Instant::now();
        }
    }
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
