//! 为五项任务各启动一个不重叠循环，并在统一截止时间后显式 abort、排空 JoinSet。

use std::time::Duration;

use anyhow::{Result, bail};
use cloud_maintenance::{MaintenanceTask, RunOutcome, RunTrigger};
use tokio::{sync::watch, task::JoinSet, time::Instant};
use tracing::{info, warn};

use super::{Runner, ShutdownSignal, control::wait_for_request, runner::SupervisedRun};

const LOCK_RETRY_MAX: Duration = Duration::from_secs(60);

pub struct Supervisor {
    tasks: JoinSet<Result<(), ()>>,
    shutdown_sender: watch::Sender<ShutdownSignal>,
    unexpected_exit: bool,
    unsettled_active_run: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RunSafety {
    pending_uncertain_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunEvidence {
    None,
    UncertainActive,
    Settled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RetryWait {
    Elapsed,
    Shutdown,
}

impl Supervisor {
    #[must_use]
    pub fn start(runner: Runner) -> Self {
        let (shutdown_sender, shutdown) = watch::channel(ShutdownSignal::Running);
        let mut tasks = JoinSet::new();
        for task in MaintenanceTask::ALL {
            tasks.spawn(run_loop(runner.clone(), task, shutdown.clone()));
        }
        Self {
            tasks,
            shutdown_sender,
            unexpected_exit: false,
            unsettled_active_run: false,
        }
    }

    pub fn request_shutdown(&self, deadline: Instant) {
        self.shutdown_sender
            .send_if_modified(|current| match current {
                ShutdownSignal::Running => {
                    *current = ShutdownSignal::Requested(deadline);
                    true
                }
                ShutdownSignal::Requested(current_deadline) if deadline < *current_deadline => {
                    *current_deadline = deadline;
                    true
                }
                ShutdownSignal::Requested(_) => false,
            });
    }

    // 正常服务期只允许这个可变借用轮询 JoinSet，select 结束后才转入消费型关停。
    pub async fn wait_for_unexpected_exit(&mut self) {
        let joined = self.tasks.join_next().await;
        self.unexpected_exit = true;
        if matches!(joined, Some(Ok(Err(())))) {
            self.unsettled_active_run = true;
        }
        match joined {
            Some(Err(_)) => warn!("维护任务循环在服务期异常退出"),
            Some(Ok(_)) | None => warn!("维护任务循环在服务期提前退出"),
        }
    }

    pub async fn shutdown_until(mut self, deadline: Instant) -> Result<()> {
        self.request_shutdown(deadline);
        loop {
            match tokio::time::timeout_at(deadline, self.tasks.join_next()).await {
                Ok(Some(Ok(Ok(())))) => {}
                Ok(Some(Ok(Err(())))) => {
                    self.unsettled_active_run = true;
                    warn!("维护任务关停时存在未收敛的活动运行");
                }
                Ok(Some(Err(_))) => {
                    self.unexpected_exit = true;
                    warn!("维护任务循环异常退出");
                }
                Ok(None) => return self.finish_result(),
                Err(_) => break,
            }
        }
        self.tasks.abort_all();
        while self.tasks.join_next().await.is_some() {}
        bail!("维护 supervisor 未在退出时限内完成，已中止剩余任务")
    }

    fn finish_result(&self) -> Result<()> {
        if self.unsettled_active_run {
            bail!("维护 supervisor 关停时存在未收敛的活动运行")
        }
        if self.unexpected_exit {
            bail!("维护 supervisor 运行期任务意外退出")
        }
        Ok(())
    }
}

async fn run_loop(
    runner: Runner,
    task: MaintenanceTask,
    mut shutdown: watch::Receiver<ShutdownSignal>,
) -> Result<(), ()> {
    let mut trigger = RunTrigger::Startup;
    let mut safety = RunSafety::default();
    loop {
        if shutdown.borrow().deadline().is_some() {
            return safety.shutdown_result();
        }
        let interval = runner.schedule(task).interval;
        let execution = runner
            .run_once_supervised(task, trigger, shutdown.clone())
            .await;
        safety.observe(run_evidence(&execution));
        if shutdown.borrow().deadline().is_some() {
            return safety.shutdown_result();
        }
        let wait = match execution.result {
            Ok(record) => {
                info!(task = %task, outcome = record.outcome.as_str(), "维护任务本轮结束");
                next_wait(interval, Some(record.outcome))
            }
            Err(_) => {
                warn!(task = %task, "维护任务 runner 失败，短周期后重试");
                next_wait(interval, None)
            }
        };
        trigger = RunTrigger::Scheduled;
        if wait_for_retry(wait, &mut shutdown).await == RetryWait::Shutdown {
            return safety.shutdown_result();
        }
    }
}

impl RunSafety {
    fn observe(&mut self, evidence: RunEvidence) {
        match evidence {
            RunEvidence::UncertainActive => self.pending_uncertain_run = true,
            RunEvidence::Settled => self.pending_uncertain_run = false,
            RunEvidence::None => {}
        }
    }

    const fn shutdown_result(self) -> Result<(), ()> {
        if self.pending_uncertain_run {
            Err(())
        } else {
            Ok(())
        }
    }
}

fn run_evidence(execution: &SupervisedRun) -> RunEvidence {
    match &execution.result {
        Err(_) => failed_run_evidence(execution.may_have_active_run),
        Ok(record) => settled_outcome_evidence(record.outcome),
    }
}

const fn failed_run_evidence(may_have_active_run: bool) -> RunEvidence {
    if may_have_active_run {
        RunEvidence::UncertainActive
    } else {
        RunEvidence::None
    }
}

const fn settled_outcome_evidence(outcome: RunOutcome) -> RunEvidence {
    match outcome {
        // 非 skipped 终态只会在同任务持锁恢复旧运行并成功写入本轮终态后返回。
        RunOutcome::Succeeded
        | RunOutcome::Failed
        | RunOutcome::TimedOut
        | RunOutcome::Cancelled
        | RunOutcome::Interrupted => RunEvidence::Settled,
        RunOutcome::Running | RunOutcome::SkippedLocked => RunEvidence::None,
    }
}

async fn wait_for_retry(
    wait: Duration,
    shutdown: &mut watch::Receiver<ShutdownSignal>,
) -> RetryWait {
    tokio::select! {
        () = tokio::time::sleep(wait) => RetryWait::Elapsed,
        _ = wait_for_request(shutdown) => RetryWait::Shutdown,
    }
}

fn next_wait(interval: Duration, outcome: Option<RunOutcome>) -> Duration {
    if outcome.is_none() || outcome == Some(RunOutcome::SkippedLocked) {
        interval.min(LOCK_RETRY_MAX)
    } else {
        interval
    }
}

#[cfg(test)]
#[path = "supervisor_tests.rs"]
mod tests;
