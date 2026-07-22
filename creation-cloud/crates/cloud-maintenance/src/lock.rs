//! 用专用 PostgreSQL 连接持有单项维护任务的 session advisory lock。
//! 连接一经取出即标记为关闭后归还，取消或 abort 时不会把锁泄漏回连接池。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use sqlx::{Connection, PgConnection, Postgres, pool::PoolConnection};

use crate::{MaintenanceTask, connection_bounds::LOCK_WAIT_TIMEOUT_SECONDS};

const SESSION_STATEMENT_TIMEOUT_SECONDS: u64 = 25;
const SESSION_IDLE_TRANSACTION_TIMEOUT_SECONDS: u64 = 25;

pub struct AdvisoryLock {
    connection: Option<PoolConnection<Postgres>>,
    namespace: i32,
    task_id: i32,
}

impl AdvisoryLock {
    pub async fn try_acquire(pool: &PgPool, task: MaintenanceTask) -> AppResult<Option<Self>> {
        let mut connection = pool
            .acquire()
            .await
            .map_err(|_| AppError::Storage("无法取得维护任务锁连接".to_owned()))?;
        // 任务 future 被取消时 Drop 无法异步解锁，因此必须关闭连接让 PostgreSQL 释放 session lock。
        connection.close_on_drop();
        configure_session(&mut connection).await?;
        let (namespace, task_id) = task.advisory_lock_identity();
        let acquired = sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_lock($1, $2)")
            .bind(namespace)
            .bind(task_id)
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| AppError::Storage("无法申请维护任务锁".to_owned()))?;
        if !acquired {
            return Ok(None);
        }
        Ok(Some(Self {
            connection: Some(connection),
            namespace,
            task_id,
        }))
    }

    #[must_use]
    pub fn connection(&mut self) -> &mut PgConnection {
        self.connection
            .as_deref_mut()
            .expect("维护任务锁连接在释放前必须存在")
    }

    pub async fn release(mut self) -> AppResult<()> {
        let Some(mut connection) = self.connection.take() else {
            return Ok(());
        };
        let unlocked = sqlx::query_scalar::<_, bool>("SELECT pg_advisory_unlock($1, $2)")
            .bind(self.namespace)
            .bind(self.task_id)
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| AppError::Storage("无法释放维护任务锁".to_owned()))?;
        let close_succeeded = connection.close().await.is_ok();
        finish_release(unlocked, close_succeeded)
    }

    pub async fn discard(mut self) {
        let Some(connection) = self.connection.take() else {
            return;
        };
        // SQL future 被取消后协议状态未知，只关闭本进程实际持有的 socket；禁止按 PID
        // 向 PostgreSQL backend 发信号。旧会话真正退出前 advisory lock 会继续阻止下一轮。
        let connection = connection.detach();
        let _ = connection.close_hard().await;
    }
}

fn finish_release(unlocked: bool, close_succeeded: bool) -> AppResult<()> {
    if !unlocked {
        return Err(AppError::Storage("维护任务锁身份已经丢失".to_owned()));
    }
    // unlock=true 已证明会话锁释放；close 已 consume close_on_drop 连接，失败也绝不回池复用。
    let _ = close_succeeded;
    Ok(())
}

async fn configure_session(connection: &mut PgConnection) -> AppResult<()> {
    for (name, seconds) in [
        ("lock_timeout", LOCK_WAIT_TIMEOUT_SECONDS),
        ("statement_timeout", SESSION_STATEMENT_TIMEOUT_SECONDS),
        (
            "idle_in_transaction_session_timeout",
            SESSION_IDLE_TRANSACTION_TIMEOUT_SECONDS,
        ),
    ] {
        let value = format!("{seconds}s");
        sqlx::query_scalar::<_, String>("SELECT set_config($1, $2, false)")
            .bind(name)
            .bind(&value)
            .fetch_one(&mut *connection)
            .await
            .map_err(|_| AppError::Storage("无法设置维护任务连接超时".to_owned()))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "lock_tests.rs"]
mod tests;
