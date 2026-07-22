//! 统一执行单次维护任务，并把锁、业务批次、状态与终态收敛绑定到同一数据库 backend。

mod budget;
mod skipped;
mod terminal;

use std::time::Duration;

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_maintenance::{
    AdvisoryLock, ErrorCode, MaintenanceTask, RunCompletion, RunOutcome, RunRecord, RunStart,
    RunTrigger, TaskExecutionReport,
};
use sqlx::PgConnection;
use tokio::{sync::watch, time::Instant};
use uuid::Uuid;

use self::budget::RunBudget;
use crate::{
    maintenance::{
        ShutdownSignal,
        adapters::{self, TaskContext},
        control::{PhaseResult, run_until},
        progress::CommittedProgress,
    },
    services::AppServices,
};

#[derive(Clone)]
pub struct Runner {
    services: AppServices,
    config: cloud_config::MaintenanceConfig,
    instance_id: Uuid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConnectionDisposition {
    Reuse,
    DiscardAndReacquire,
}

struct ExecutionResult {
    outcome: RunOutcome,
    error: Option<ErrorCode>,
    report: TaskExecutionReport,
    connection: ConnectionDisposition,
    terminal_deadline: Instant,
    shutdown_observed: bool,
}

#[derive(Default)]
struct RunLifecycle {
    may_have_active_run: bool,
}

pub(super) struct SupervisedRun {
    pub(super) result: AppResult<RunRecord>,
    pub(super) may_have_active_run: bool,
}

impl Runner {
    #[must_use]
    pub fn new(services: AppServices, config: cloud_config::MaintenanceConfig) -> Self {
        Self {
            services,
            config,
            instance_id: Uuid::now_v7(),
        }
    }

    #[must_use]
    pub const fn schedule(&self, task: MaintenanceTask) -> cloud_config::TaskSchedule {
        match task {
            MaintenanceTask::ExpiredSessions => self.config.expired_sessions,
            MaintenanceTask::SyncRetention => self.config.sync_retention,
            MaintenanceTask::DownloadAggregation => self.config.download_aggregation,
            MaintenanceTask::PublishedAssetInspection => self.config.published_asset_inspection,
            MaintenanceTask::BackupFreshness => self.config.backup_freshness,
        }
    }

    pub async fn run_once(
        &self,
        task: MaintenanceTask,
        trigger: RunTrigger,
        shutdown: watch::Receiver<ShutdownSignal>,
    ) -> AppResult<RunRecord> {
        let mut lifecycle = RunLifecycle::default();
        self.run_once_inner(task, trigger, shutdown, &mut lifecycle)
            .await
    }

    pub(super) async fn run_once_supervised(
        &self,
        task: MaintenanceTask,
        trigger: RunTrigger,
        shutdown: watch::Receiver<ShutdownSignal>,
    ) -> SupervisedRun {
        let mut lifecycle = RunLifecycle::default();
        let result = self
            .run_once_inner(task, trigger, shutdown, &mut lifecycle)
            .await;
        SupervisedRun {
            result,
            may_have_active_run: lifecycle.may_have_active_run,
        }
    }

    async fn run_once_inner(
        &self,
        task: MaintenanceTask,
        trigger: RunTrigger,
        mut shutdown: watch::Receiver<ShutdownSignal>,
        lifecycle: &mut RunLifecycle,
    ) -> AppResult<RunRecord> {
        let budget = RunBudget::new(self.schedule(task).timeout);
        let lock_result = match run_until(
            &mut shutdown,
            budget.execution_deadline,
            self.services.maintenance.try_lock(task),
        )
        .await
        {
            PhaseResult::Completed(result) => result?,
            PhaseResult::Cancelled(_) => return Err(cancelled_before_start()),
            PhaseResult::TimedOut => return Err(timed_out_before_start()),
        };
        let Some(mut lock) = lock_result else {
            return self
                .record_skipped_attempt(task, trigger, budget, &mut shutdown)
                .await;
        };

        let context = match run_until(
            &mut shutdown,
            budget.execution_deadline,
            self.context_on(task, lock.connection()),
        )
        .await
        {
            PhaseResult::Completed(result) => result?,
            PhaseResult::Cancelled(deadline) => {
                discard_before_start(lock, &shutdown, deadline).await;
                return Err(cancelled_before_start());
            }
            PhaseResult::TimedOut => {
                discard_before_start(lock, &shutdown, budget.terminal_deadline).await;
                return Err(timed_out_before_start());
            }
        };
        let start = self.run_start(task, trigger, &context);
        match run_until(
            &mut shutdown,
            budget.execution_deadline,
            self.services
                .maintenance
                .recover_interrupted_on(lock.connection(), task),
        )
        .await
        {
            PhaseResult::Completed(result) => {
                result?;
            }
            PhaseResult::Cancelled(deadline) => {
                discard_before_start(lock, &shutdown, deadline).await;
                return Err(cancelled_before_start());
            }
            PhaseResult::TimedOut => {
                discard_before_start(lock, &shutdown, budget.terminal_deadline).await;
                return Err(timed_out_before_start());
            }
        }
        // start 的响应丢失时可能已经提交；只有后续探测证明 run_id 不存在才能清除此标记。
        lifecycle.may_have_active_run = true;
        let start_result = run_until(
            &mut shutdown,
            budget.execution_deadline,
            self.services
                .maintenance
                .start_run_on(lock.connection(), &start),
        )
        .await;
        match start_result {
            PhaseResult::Completed(Ok(())) => {}
            PhaseResult::Completed(Err(error)) => {
                return self
                    .resolve_uncertain_start(
                        lock,
                        &start,
                        RunOutcome::Failed,
                        ErrorCode::TaskFailed,
                        &mut shutdown,
                        budget.terminal_deadline,
                        false,
                        error,
                        lifecycle,
                    )
                    .await;
            }
            PhaseResult::Cancelled(deadline) => {
                return self
                    .resolve_uncertain_start(
                        lock,
                        &start,
                        RunOutcome::Cancelled,
                        ErrorCode::Cancelled,
                        &mut shutdown,
                        deadline,
                        true,
                        cancelled_before_start(),
                        lifecycle,
                    )
                    .await;
            }
            PhaseResult::TimedOut => {
                return self
                    .resolve_uncertain_start(
                        lock,
                        &start,
                        RunOutcome::TimedOut,
                        ErrorCode::TimedOut,
                        &mut shutdown,
                        budget.terminal_deadline,
                        false,
                        timed_out_before_start(),
                        lifecycle,
                    )
                    .await;
            }
        }

        let progress = CommittedProgress::default();
        let execution = self
            .execute_with_limits(
                task,
                budget,
                &mut shutdown,
                &context,
                lock.connection(),
                &progress,
            )
            .await;
        let completion = completion(&start, execution.outcome, execution.error, execution.report);
        self.settle_known_run(
            lock,
            completion,
            &mut shutdown,
            execution.terminal_deadline,
            execution.connection == ConnectionDisposition::DiscardAndReacquire,
            execution.shutdown_observed,
        )
        .await
    }

    async fn execute_with_limits(
        &self,
        task: MaintenanceTask,
        budget: RunBudget,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        context: &TaskContext,
        connection: &mut PgConnection,
        progress: &CommittedProgress,
    ) -> ExecutionResult {
        let execution = adapters::execute(
            task,
            &self.services,
            &self.config,
            context,
            connection,
            progress,
        );
        match run_until(shutdown, budget.execution_deadline, execution).await {
            PhaseResult::Completed(result) => match result {
                Ok(report) => terminal(
                    RunOutcome::Succeeded,
                    None,
                    report,
                    ConnectionDisposition::Reuse,
                    budget.terminal_deadline,
                    false,
                ),
                Err(_) => terminal(
                    RunOutcome::Failed,
                    Some(ErrorCode::TaskFailed),
                    progress.snapshot(),
                    ConnectionDisposition::Reuse,
                    budget.terminal_deadline,
                    false,
                ),
            },
            PhaseResult::Cancelled(deadline) => terminal(
                RunOutcome::Cancelled,
                Some(ErrorCode::Cancelled),
                progress.snapshot(),
                ConnectionDisposition::DiscardAndReacquire,
                deadline,
                true,
            ),
            PhaseResult::TimedOut => terminal(
                RunOutcome::TimedOut,
                Some(ErrorCode::TimedOut),
                progress.snapshot(),
                ConnectionDisposition::DiscardAndReacquire,
                budget.terminal_deadline,
                false,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn resolve_uncertain_start(
        &self,
        lock: AdvisoryLock,
        start: &RunStart,
        outcome: RunOutcome,
        error_code: ErrorCode,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
        deadline: Instant,
        shutdown_observed: bool,
        absent_error: AppError,
        lifecycle: &mut RunLifecycle,
    ) -> AppResult<RunRecord> {
        let completion = completion(
            start,
            outcome,
            Some(error_code),
            TaskExecutionReport::default(),
        );
        match self
            .settle_uncertain_start(lock, completion, shutdown, deadline, shutdown_observed)
            .await?
        {
            Some(record) => Ok(record),
            None => {
                lifecycle.may_have_active_run = false;
                Err(absent_error)
            }
        }
    }

    async fn context_on(
        &self,
        task: MaintenanceTask,
        connection: &mut PgConnection,
    ) -> AppResult<TaskContext> {
        let now = self
            .services
            .maintenance
            .database_now_on(connection)
            .await?;
        self.context_from_now(task, now)
    }

    fn context_from_now(
        &self,
        task: MaintenanceTask,
        now: DateTime<Utc>,
    ) -> AppResult<TaskContext> {
        let (cutoff_at, active_cutoff_at) = match task {
            MaintenanceTask::ExpiredSessions => (
                Some(subtract(now, self.config.expired_session_retention)?),
                None,
            ),
            MaintenanceTask::SyncRetention => (
                Some(subtract(now, self.config.sync_retention_window)?),
                Some(subtract(now, self.config.sync_active_window)?),
            ),
            _ => (None, None),
        };
        Ok(TaskContext {
            now,
            cutoff_at,
            active_cutoff_at,
        })
    }

    fn run_start(
        &self,
        task: MaintenanceTask,
        trigger: RunTrigger,
        context: &TaskContext,
    ) -> RunStart {
        RunStart {
            run_id: Uuid::now_v7(),
            task,
            trigger,
            instance_id: self.instance_id,
            cutoff_at: context.cutoff_at,
            active_cutoff_at: context.active_cutoff_at,
        }
    }
}

fn terminal(
    outcome: RunOutcome,
    error: Option<ErrorCode>,
    mut report: TaskExecutionReport,
    connection: ConnectionDisposition,
    terminal_deadline: Instant,
    shutdown_observed: bool,
) -> ExecutionResult {
    if outcome != RunOutcome::Succeeded {
        report.observation = None;
    }
    ExecutionResult {
        outcome,
        error,
        report,
        connection,
        terminal_deadline,
        shutdown_observed,
    }
}

fn completion(
    start: &RunStart,
    outcome: RunOutcome,
    error: Option<ErrorCode>,
    report: TaskExecutionReport,
) -> RunCompletion {
    RunCompletion {
        run_id: start.run_id,
        task: start.task,
        outcome,
        observation: report.observation,
        error,
        report,
    }
}

async fn discard_before_start(
    lock: AdvisoryLock,
    shutdown: &watch::Receiver<ShutdownSignal>,
    deadline: Instant,
) {
    let deadline = shutdown
        .borrow()
        .deadline()
        .map_or(deadline, |requested| requested.min(deadline));
    let _ = tokio::time::timeout_at(deadline, lock.discard()).await;
}

fn cancelled_before_start() -> AppError {
    AppError::Unavailable("维护任务在创建运行记录前已取消".to_owned())
}

fn timed_out_before_start() -> AppError {
    AppError::Unavailable("维护任务在创建运行记录前已达到执行截止时间".to_owned())
}

fn subtract(now: DateTime<Utc>, duration: Duration) -> AppResult<DateTime<Utc>> {
    let duration = chrono::Duration::from_std(duration)
        .map_err(|_| AppError::Internal("维护任务时间窗口超出范围".to_owned()))?;
    now.checked_sub_signed(duration)
        .ok_or_else(|| AppError::Internal("维护任务 cutoff 超出时间范围".to_owned()))
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "runner/postgres_tests.rs"]
mod postgres_tests;
