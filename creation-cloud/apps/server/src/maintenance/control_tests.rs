//! 验证统一取消信号不会轮询已取消阶段，并会丢弃正在等待的 future。

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

use super::*;

struct PendingDrop(Arc<AtomicBool>);

impl Future for PendingDrop {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for PendingDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn initial_shutdown_never_polls_the_operation() {
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    let (_sender, mut shutdown) = watch::channel(ShutdownSignal::Requested(deadline));
    let polled = Arc::new(AtomicBool::new(false));
    let operation_polled = polled.clone();
    let operation = std::future::poll_fn(move |_| {
        operation_polled.store(true, Ordering::Relaxed);
        std::task::Poll::<()>::Pending
    });

    let result = run_until(&mut shutdown, deadline, operation).await;
    assert!(matches!(result, PhaseResult::Cancelled(value) if value == deadline));
    assert!(!polled.load(Ordering::Relaxed));
}

#[tokio::test]
async fn closed_control_channel_fails_closed() {
    let (sender, mut shutdown) = watch::channel(ShutdownSignal::Running);
    drop(sender);
    let result = run_until(
        &mut shutdown,
        Instant::now() + std::time::Duration::from_secs(30),
        std::future::pending::<()>(),
    )
    .await;
    assert!(matches!(result, PhaseResult::Cancelled(_)));
}

#[tokio::test]
async fn pending_acquire_like_future_is_dropped_on_live_shutdown() {
    let (sender, mut shutdown) = watch::channel(ShutdownSignal::Running);
    let deadline = Instant::now() + std::time::Duration::from_secs(30);
    let dropped = Arc::new(AtomicBool::new(false));
    let request = tokio::spawn(async move {
        tokio::task::yield_now().await;
        sender
            .send(ShutdownSignal::Requested(deadline))
            .expect("测试接收端必须存在");
    });
    let result = run_until(&mut shutdown, deadline, PendingDrop(dropped.clone())).await;
    request.await.expect("关停请求任务不应失败");
    assert!(matches!(result, PhaseResult::Cancelled(value) if value == deadline));
    assert!(dropped.load(Ordering::Relaxed));
}

#[tokio::test]
async fn absolute_deadline_drops_an_in_flight_operation() {
    let (_sender, mut shutdown) = watch::channel(ShutdownSignal::Running);
    let deadline = Instant::now() + std::time::Duration::from_millis(5);
    let dropped = Arc::new(AtomicBool::new(false));
    let result = run_until(&mut shutdown, deadline, PendingDrop(dropped.clone())).await;
    assert!(matches!(result, PhaseResult::TimedOut));
    assert!(dropped.load(Ordering::Relaxed));
}
