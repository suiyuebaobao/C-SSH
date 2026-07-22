//! 在绝对截止时间内探测并收敛运行终态，且绝不复用被中断的数据库连接。

use std::{future::Future, time::Duration};

use cloud_domain::{AppError, AppResult};
use cloud_maintenance::{
    AdvisoryLock, ErrorCode, MaintenanceTask, RunCompletion, RunOutcome, RunRecord,
};
use tokio::{sync::watch, time::Instant};

use crate::maintenance::{
    ShutdownSignal,
    control::{PhaseResult, run_until},
};

use super::Runner;

const REACQUIRE_RETRY: Duration = Duration::from_millis(50);
const MAX_PROVEN_TERMINAL_WRITES: u8 = 2;
const MAX_PROBE_ATTEMPTS: u8 = 2;

impl Runner {
    pub(super) async fn settle_known_run(
        &self,
        lock: AdvisoryLock,
        completion: RunCompletion,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        deadline: Instant,
        connection_tainted: bool,
        shutdown_observed: bool,
    ) -> AppResult<RunRecord> {
        self.settle_run(
            lock,
            completion,
            shutdown,
            deadline,
            connection_tainted,
            shutdown_observed,
            false,
        )
        .await?
        .ok_or_else(|| AppError::Storage("已开始的维护运行记录消失".to_owned()))
    }

    pub(super) async fn settle_uncertain_start(
        &self,
        lock: AdvisoryLock,
        completion: RunCompletion,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        deadline: Instant,
        shutdown_observed: bool,
    ) -> AppResult<Option<RunRecord>> {
        self.settle_run(
            lock,
            completion,
            shutdown,
            deadline,
            true,
            shutdown_observed,
            true,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn settle_run(
        &self,
        lock: AdvisoryLock,
        mut completion: RunCompletion,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        mut deadline: Instant,
        connection_tainted: bool,
        mut shutdown_observed: bool,
        mut may_be_absent: bool,
    ) -> AppResult<Option<RunRecord>> {
        let mut lock = Some(lock);
        let mut needs_probe = connection_tainted || may_be_absent;
        let mut write_attempts = 0_u8;
        let mut probe_attempts = 0_u8;
        if connection_tainted {
            discard_until(
                lock.take().expect("待淘汰锁连接必须存在"),
                effective_deadline(shutdown, deadline),
            )
            .await;
        }

        loop {
            if lock.is_none() {
                match self
                    .reacquire_until(shutdown, deadline, shutdown_observed, completion.task)
                    .await?
                {
                    PhaseResult::Completed(acquired) => lock = Some(acquired),
                    PhaseResult::Cancelled(new_deadline) => {
                        mark_cancelled(&mut completion);
                        shutdown_observed = true;
                        deadline = new_deadline;
                        continue;
                    }
                    PhaseResult::TimedOut => return Err(terminal_deadline_error("reacquire")),
                }
                needs_probe = true;
            }

            let mut current_lock = lock.take().expect("终态锁必须已经取得");
            if needs_probe {
                probe_attempts += 1;
                let read = terminal_until(
                    shutdown,
                    deadline,
                    shutdown_observed,
                    self.services
                        .maintenance
                        .run_on(current_lock.connection(), completion.run_id),
                )
                .await;
                match read {
                    PhaseResult::Completed(Ok(record)) if record.outcome != RunOutcome::Running => {
                        return release_record_until(
                            current_lock,
                            record,
                            shutdown,
                            deadline,
                            shutdown_observed,
                        )
                        .await
                        .map(Some);
                    }
                    PhaseResult::Completed(Ok(_)) => {
                        may_be_absent = false;
                    }
                    PhaseResult::Completed(Err(AppError::NotFound(_))) if may_be_absent => {
                        release_empty_until(current_lock, shutdown, deadline, shutdown_observed)
                            .await?;
                        return Ok(None);
                    }
                    PhaseResult::Completed(Err(error)) => {
                        discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                        if probe_attempts >= MAX_PROBE_ATTEMPTS {
                            return Err(error);
                        }
                        needs_probe = true;
                        continue;
                    }
                    PhaseResult::Cancelled(new_deadline) => {
                        mark_cancelled(&mut completion);
                        shutdown_observed = true;
                        deadline = new_deadline;
                        discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                        probe_attempts = 0;
                        continue;
                    }
                    PhaseResult::TimedOut => {
                        discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                        return Err(terminal_deadline_error("probe"));
                    }
                }
            }

            if write_attempts >= MAX_PROVEN_TERMINAL_WRITES {
                discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                return Err(AppError::Storage(
                    "维护运行终态写入在已证明仍为 running 后失败".to_owned(),
                ));
            }
            write_attempts += 1;
            let completed = terminal_until(
                shutdown,
                deadline,
                shutdown_observed,
                self.services
                    .maintenance
                    .complete_run_on(current_lock.connection(), &completion),
            )
            .await;
            match completed {
                PhaseResult::Completed(Ok(record)) => {
                    return release_record_until(
                        current_lock,
                        record,
                        shutdown,
                        deadline,
                        shutdown_observed,
                    )
                    .await
                    .map(Some);
                }
                PhaseResult::Completed(Err(_)) => {
                    discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                    needs_probe = true;
                    probe_attempts = 0;
                }
                PhaseResult::Cancelled(new_deadline) => {
                    mark_cancelled(&mut completion);
                    shutdown_observed = true;
                    deadline = new_deadline;
                    discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                    needs_probe = true;
                }
                PhaseResult::TimedOut => {
                    discard_until(current_lock, effective_deadline(shutdown, deadline)).await;
                    return Err(terminal_deadline_error("write"));
                }
            }
        }
    }

    async fn reacquire_until(
        &self,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        deadline: Instant,
        shutdown_observed: bool,
        task: MaintenanceTask,
    ) -> AppResult<PhaseResult<AdvisoryLock>> {
        loop {
            let attempt = terminal_until(
                shutdown,
                deadline,
                shutdown_observed,
                self.services.maintenance.try_lock(task),
            )
            .await;
            match attempt {
                PhaseResult::Completed(Ok(Some(lock))) => {
                    return Ok(PhaseResult::Completed(lock));
                }
                PhaseResult::Completed(Ok(None)) => {}
                PhaseResult::Completed(Err(error)) => return Err(error),
                PhaseResult::Cancelled(deadline) => {
                    return Ok(PhaseResult::Cancelled(deadline));
                }
                PhaseResult::TimedOut => return Ok(PhaseResult::TimedOut),
            }
            match terminal_until(
                shutdown,
                deadline,
                shutdown_observed,
                tokio::time::sleep(REACQUIRE_RETRY),
            )
            .await
            {
                PhaseResult::Completed(()) => {}
                PhaseResult::Cancelled(deadline) => {
                    return Ok(PhaseResult::Cancelled(deadline));
                }
                PhaseResult::TimedOut => return Ok(PhaseResult::TimedOut),
            }
        }
    }
}

async fn terminal_until<F>(
    shutdown: &mut watch::Receiver<ShutdownSignal>,
    deadline: Instant,
    shutdown_observed: bool,
    operation: F,
) -> PhaseResult<F::Output>
where
    F: Future,
{
    if !shutdown_observed {
        return run_until(shutdown, deadline, operation).await;
    }
    let deadline = effective_deadline(shutdown, deadline);
    if deadline <= Instant::now() {
        return PhaseResult::TimedOut;
    }
    match tokio::time::timeout_at(deadline, operation).await {
        Ok(output) => PhaseResult::Completed(output),
        Err(_) => PhaseResult::TimedOut,
    }
}

fn effective_deadline(shutdown: &watch::Receiver<ShutdownSignal>, deadline: Instant) -> Instant {
    shutdown
        .borrow()
        .deadline()
        .map_or(deadline, |requested| requested.min(deadline))
}

async fn discard_until(lock: AdvisoryLock, deadline: Instant) {
    let _ = tokio::time::timeout_at(deadline, lock.discard()).await;
}

async fn release_record_until(
    lock: AdvisoryLock,
    record: RunRecord,
    shutdown: &mut watch::Receiver<ShutdownSignal>,
    deadline: Instant,
    shutdown_observed: bool,
) -> AppResult<RunRecord> {
    match terminal_until(shutdown, deadline, shutdown_observed, lock.release()).await {
        PhaseResult::Completed(result) => result.map(|()| record),
        // release future 持有 close_on_drop 连接；截止或取消会关闭 backend，记录已经是终态。
        PhaseResult::Cancelled(_) | PhaseResult::TimedOut => Ok(record),
    }
}

async fn release_empty_until(
    lock: AdvisoryLock,
    shutdown: &mut watch::Receiver<ShutdownSignal>,
    deadline: Instant,
    shutdown_observed: bool,
) -> AppResult<()> {
    match terminal_until(shutdown, deadline, shutdown_observed, lock.release()).await {
        PhaseResult::Completed(result) => result,
        PhaseResult::Cancelled(_) | PhaseResult::TimedOut => Ok(()),
    }
}

fn mark_cancelled(completion: &mut RunCompletion) {
    completion.outcome = RunOutcome::Cancelled;
    completion.observation = None;
    completion.error = Some(ErrorCode::Cancelled);
}

fn terminal_deadline_error(phase: &'static str) -> AppError {
    AppError::Unavailable(format!("维护任务未在绝对截止时间内完成终态收敛: {phase}"))
}

#[cfg(test)]
#[path = "terminal_tests.rs"]
mod tests;
