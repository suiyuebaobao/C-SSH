//! 作为锁、运行仓储和只读状态查询的统一公开入口。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    AdvisoryLock, MaintenanceStatus, MaintenanceTask, RunCompletion, RunRecord, RunStart,
    repository,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn try_lock(&self, task: MaintenanceTask) -> AppResult<Option<AdvisoryLock>> {
        AdvisoryLock::try_acquire(&self.pool, task).await
    }

    pub async fn database_now(&self) -> AppResult<DateTime<Utc>> {
        repository::database_now(&self.pool).await
    }

    pub async fn database_now_on(&self, connection: &mut PgConnection) -> AppResult<DateTime<Utc>> {
        repository::database_now_on(connection).await
    }

    pub async fn recover_interrupted_on(
        &self,
        connection: &mut PgConnection,
        task: MaintenanceTask,
    ) -> AppResult<u64> {
        repository::recover_interrupted(connection, task).await
    }

    pub async fn start_run_on(
        &self,
        connection: &mut PgConnection,
        run: &RunStart,
    ) -> AppResult<()> {
        repository::start(connection, run).await
    }

    pub async fn complete_run_on(
        &self,
        connection: &mut PgConnection,
        completion: &RunCompletion,
    ) -> AppResult<RunRecord> {
        repository::complete(connection, completion).await
    }

    pub async fn record_skipped_locked(&self, run: &RunStart) -> AppResult<RunRecord> {
        repository::record_skipped_locked(&self.pool, run).await
    }

    pub async fn record_skipped_locked_on(
        &self,
        connection: &mut PgConnection,
        run: &RunStart,
    ) -> AppResult<RunRecord> {
        repository::record_skipped_locked_on(connection, run).await
    }

    pub async fn run(&self, run_id: Uuid) -> AppResult<RunRecord> {
        repository::read_run(&self.pool, run_id).await
    }

    pub async fn run_on(
        &self,
        connection: &mut PgConnection,
        run_id: Uuid,
    ) -> AppResult<RunRecord> {
        repository::read_run_on(connection, run_id).await
    }

    pub async fn status(&self, task: MaintenanceTask) -> AppResult<MaintenanceStatus> {
        repository::status(&self.pool, task).await
    }

    pub async fn status_on(
        &self,
        connection: &mut PgConnection,
        task: MaintenanceTask,
    ) -> AppResult<MaintenanceStatus> {
        repository::status_on(connection, task).await
    }

    pub async fn statuses(&self) -> AppResult<Vec<MaintenanceStatus>> {
        let mut statuses = Vec::with_capacity(MaintenanceTask::ALL.len());
        for task in MaintenanceTask::ALL {
            statuses.push(repository::status(&self.pool, task).await?);
        }
        Ok(statuses)
    }
}
