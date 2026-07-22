//! 用一次性 close_on_drop 数据库会话记录未获锁尝试，取消后绝不归还该连接。

use cloud_domain::{AppError, AppResult};
use cloud_maintenance::{MaintenanceTask, RunRecord, RunTrigger};
use sqlx::{Postgres, pool::PoolConnection};
use tokio::{sync::watch, time::Instant};

use crate::maintenance::{
    ShutdownSignal,
    control::{PhaseResult, run_until},
};

use super::{RunBudget, Runner, cancelled_before_start, timed_out_before_start};

impl Runner {
    pub(super) async fn record_skipped_attempt(
        &self,
        task: MaintenanceTask,
        trigger: RunTrigger,
        budget: RunBudget,
        shutdown: &mut watch::Receiver<ShutdownSignal>,
    ) -> AppResult<RunRecord> {
        let acquired = run_until(
            shutdown,
            budget.execution_deadline,
            self.services.pool.acquire(),
        )
        .await;
        let mut connection = match acquired {
            PhaseResult::Completed(Ok(connection)) => connection,
            PhaseResult::Completed(Err(_)) => {
                return Err(AppError::Storage(
                    "无法取得维护 skipped 记录连接".to_owned(),
                ));
            }
            PhaseResult::Cancelled(_) => return Err(cancelled_before_start()),
            PhaseResult::TimedOut => return Err(timed_out_before_start()),
        };
        // 从这一行起任何 select!/abort 都只会关闭 backend，不会把未知协议状态归还池。
        connection.close_on_drop();

        let context = run_until(
            shutdown,
            budget.execution_deadline,
            self.context_on(task, &mut connection),
        )
        .await;
        let context = match context {
            PhaseResult::Completed(Ok(context)) => context,
            PhaseResult::Completed(Err(error)) => {
                close_until(connection, budget.terminal_deadline).await;
                return Err(error);
            }
            PhaseResult::Cancelled(deadline) => {
                close_until(connection, deadline).await;
                return Err(cancelled_before_start());
            }
            PhaseResult::TimedOut => {
                close_until(connection, budget.terminal_deadline).await;
                return Err(timed_out_before_start());
            }
        };
        let start = self.run_start(task, trigger, &context);
        let recorded = run_until(
            shutdown,
            budget.execution_deadline,
            self.services
                .maintenance
                .record_skipped_locked_on(&mut connection, &start),
        )
        .await;
        match recorded {
            PhaseResult::Completed(result) => {
                close_until(connection, budget.terminal_deadline).await;
                result
            }
            PhaseResult::Cancelled(deadline) => {
                close_until(connection, deadline).await;
                Err(cancelled_before_start())
            }
            PhaseResult::TimedOut => {
                close_until(connection, budget.terminal_deadline).await;
                Err(timed_out_before_start())
            }
        }
    }
}

async fn close_until(connection: PoolConnection<Postgres>, deadline: Instant) {
    let _ = tokio::time::timeout_at(deadline, connection.close()).await;
}
