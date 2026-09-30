//! 在随机独占 schema 的真实 PostgreSQL 上验证 runner 取消、连接淘汰与绝对截止时间。
//! 每项测试都运行正式迁移、使用 max=1 任务连接池，并只清理本轮生成的 schema。

use std::time::Duration;

use cloud_maintenance::{
    AdvisoryLock, ErrorCode, MaintenanceTask, RunCompletion, RunOutcome, RunStart, RunTrigger,
    Service, TaskExecutionReport,
};
use tokio::{sync::watch, time::Instant};
use uuid::Uuid;

use super::Runner;
use crate::maintenance::ShutdownSignal;

const DATABASE_URL_ENV: &str = "CLOUD_DATABASE_URL";
const OBSERVE_TIMEOUT: Duration = Duration::from_secs(5);

// PostgreSQL advisory lock 的作用域是整个数据库，随机 schema 无法隔离同任务并发测试。
static SERIAL_POSTGRES_TESTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[path = "postgres_support.rs"]
mod support;

use support::{SchemaGuard, TestDatabase, TestResult, boxed, migrate_schema, require};

macro_rules! real_postgres_test {
    ($name:ident, $scenario:ident) => {
        #[tokio::test]
        #[ignore = "需要由 no-mock 门禁显式提供隔离真实 PostgreSQL"]
        async fn $name() {
            let _serial_guard = SERIAL_POSTGRES_TESTS.lock().await;
            let database_url = std::env::var(DATABASE_URL_ENV)
                .expect("no-mock 测试必须显式设置 CLOUD_DATABASE_URL");
            let mut schema = SchemaGuard::create(&database_url)
                .await
                .expect("必须创建随机独占 PostgreSQL schema");
            let result: TestResult = async {
                migrate_schema(&database_url, schema.name()).await?;
                let database = TestDatabase::connect(&database_url, schema.name()).await?;
                let scenario = $scenario(&database).await;
                database.close().await;
                scenario
            }
            .await;
            schema
                .cleanup()
                .await
                .expect("必须只清理本轮随机独占 schema");
            result.expect("runner 真实 PostgreSQL 取消契约必须成立");
        }
    };
}

real_postgres_test!(
    initial_cancellation_does_not_start_a_run,
    verify_initial_cancel
);
real_postgres_test!(
    pool_wait_is_cancelled_without_reusing_a_session,
    verify_pool_wait_cancel
);
real_postgres_test!(
    unknown_start_is_probed_before_terminal_write,
    verify_unknown_start
);
real_postgres_test!(
    blocked_business_sql_cancellation_becomes_cancelled,
    verify_business_cancel
);
real_postgres_test!(
    hard_deadline_leaves_recoverable_interrupted_run,
    verify_hard_deadline
);
real_postgres_test!(
    non_lock_sql_past_deadline_keeps_truthful_running_state,
    verify_non_lock_hard_deadline
);

async fn verify_initial_cancel(database: &TestDatabase) -> TestResult {
    let runner = database.runner();
    let deadline = Instant::now() + Duration::from_secs(5);
    let (_sender, shutdown) = watch::channel(ShutdownSignal::Requested(deadline));
    let result = runner
        .run_once(
            MaintenanceTask::BackupFreshness,
            RunTrigger::Manual,
            shutdown,
        )
        .await;
    require(result.is_err(), "初始取消不得返回运行记录")?;
    require(database.run_count().await? == 0, "初始取消不得写入运行记录")?;
    assert_lock_available(database, MaintenanceTask::BackupFreshness).await
}

async fn verify_pool_wait_cancel(database: &TestDatabase) -> TestResult {
    let held = database.runner_pool.acquire().await?;
    let runner = database.runner();
    let (sender, shutdown) = watch::channel(ShutdownSignal::Running);
    let attempt: TestResult<_> = async {
        let future = runner.run_once(
            MaintenanceTask::BackupFreshness,
            RunTrigger::Manual,
            shutdown,
        );
        tokio::pin!(future);
        // 先精确轮询一次，使 run_once 停在 max=1 连接池等待，而不是依赖定时 sleep 猜测。
        tokio::select! {
            biased;
            result = &mut future => return Err(boxed(format!(
                "独占连接仍被持有时 runner 不得提前结束：{}",
                result.is_ok()
            ))),
            () = std::future::ready(()) => {}
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        sender
            .send(ShutdownSignal::Requested(deadline))
            .map_err(|_| boxed("无法发送连接池等待取消信号"))?;
        tokio::time::timeout_at(deadline, &mut future)
            .await
            .map_err(|_| boxed("连接池等待未在绝对截止时间内取消"))
    }
    .await;
    drop(held);
    let result = attempt?;
    require(result.is_err(), "连接池等待取消不得创建运行记录")?;
    require(
        database.run_count().await? == 0,
        "池等待取消不得写入运行记录",
    )
}

async fn verify_unknown_start(database: &TestDatabase) -> TestResult {
    let runner = database.runner();
    let task = MaintenanceTask::DownloadAggregation;
    let deadline = Instant::now() + Duration::from_secs(10);
    let (_sender, mut shutdown) = watch::channel(ShutdownSignal::Requested(deadline));

    let absent = run_start(task);
    let absent_lock = acquire_lock(&runner, task).await?;
    let absent_result = runner
        .settle_uncertain_start(
            absent_lock,
            cancelled_completion(&absent),
            &mut shutdown,
            deadline,
            true,
        )
        .await?;
    require(absent_result.is_none(), "不存在的 run_id 必须判定为未开始")?;

    let committed = run_start(task);
    let mut committed_lock = acquire_lock(&runner, task).await?;
    runner
        .services
        .maintenance
        .start_run_on(committed_lock.connection(), &committed)
        .await?;
    let record = runner
        .settle_uncertain_start(
            committed_lock,
            cancelled_completion(&committed),
            &mut shutdown,
            deadline,
            true,
        )
        .await?
        .ok_or_else(|| boxed("已提交 start 的 run_id 不得被判定为不存在"))?;
    require(
        record.outcome == RunOutcome::Cancelled && record.error == Some(ErrorCode::Cancelled),
        "已提交但结果未知的 start 必须经探测收敛为 cancelled",
    )?;
    require(
        database.running_count(task).await? == 0,
        "探测收敛后不得遗留 running",
    )?;
    assert_lock_available(database, task).await
}

async fn verify_business_cancel(database: &TestDatabase) -> TestResult {
    let task = MaintenanceTask::ExpiredSessions;
    let mut blocker = database.observer.begin().await?;
    sqlx::query("LOCK TABLE sessions IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *blocker)
        .await?;
    let runner = database.runner();
    let (sender, shutdown) = watch::channel(ShutdownSignal::Running);
    let attempt: TestResult<_> = async {
        let future = runner.run_once(task, RunTrigger::Manual, shutdown);
        tokio::pin!(future);
        tokio::select! {
            biased;
            _ = &mut future => return Err(boxed("业务 SQL 被表锁阻塞前 runner 不得结束")),
            observed = wait_for_runner_lock(database) => observed?,
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        sender
            .send(ShutdownSignal::Requested(deadline))
            .map_err(|_| boxed("无法发送业务 SQL 取消信号"))?;
        tokio::time::timeout_at(deadline, &mut future)
            .await
            .map_err(|_| boxed("业务 SQL 取消未在绝对截止时间内收敛"))
    }
    .await;
    blocker.rollback().await?;
    let record = attempt??;
    require(
        record.outcome == RunOutcome::Cancelled && record.error == Some(ErrorCode::Cancelled),
        "真实业务 SQL 取消必须写入 cancelled 终态",
    )?;
    require(
        database.running_count(task).await? == 0,
        "业务取消后不得遗留 running",
    )?;
    assert_lock_available(database, task).await
}

async fn verify_hard_deadline(database: &TestDatabase) -> TestResult {
    let task = MaintenanceTask::ExpiredSessions;
    let mut business_blocker = database.observer.begin().await?;
    sqlx::query("LOCK TABLE sessions IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *business_blocker)
        .await?;
    let runner = database.runner();
    let (sender, shutdown) = watch::channel(ShutdownSignal::Running);
    let mut terminal_blocker = None;
    let attempt: TestResult<_> = async {
        let future = runner.run_once(task, RunTrigger::Manual, shutdown);
        tokio::pin!(future);
        tokio::select! {
            biased;
            _ = &mut future => return Err(boxed("硬截止测试的业务 SQL 不得提前结束")),
            observed = wait_for_runner_lock(database) => observed?,
        }
        let mut blocker = database.observer.begin().await?;
        sqlx::query("SELECT task_name FROM maintenance_task_state WHERE task_name = $1 FOR UPDATE")
            .bind(task.as_str())
            .fetch_one(&mut *blocker)
            .await?;
        terminal_blocker = Some(blocker);
        let deadline = Instant::now() + Duration::from_secs(1);
        sender
            .send(ShutdownSignal::Requested(deadline))
            .map_err(|_| boxed("无法发送硬截止取消信号"))?;
        tokio::time::timeout_at(deadline + Duration::from_secs(2), &mut future)
            .await
            .map_err(|_| boxed("runner 超过绝对截止时间及测试容差仍未退出"))
    }
    .await;
    if let Some(blocker) = terminal_blocker {
        blocker.rollback().await?;
    }
    business_blocker.rollback().await?;
    let result = attempt?;
    require(result.is_err(), "终态行锁跨过硬截止时 runner 必须失败退出")?;

    let mut recovery_lock = eventually_acquire_lock(database, task).await?;
    let recovered = Service::new(database.observer.clone())
        .recover_interrupted_on(recovery_lock.connection(), task)
        .await?;
    require(recovered == 1, "下一持锁者必须恢复一个硬截止遗留运行")?;
    let record = Service::new(database.observer.clone())
        .run_on(
            recovery_lock.connection(),
            database.only_run_id(task).await?,
        )
        .await?;
    require(
        record.outcome == RunOutcome::Interrupted && record.error == Some(ErrorCode::Interrupted),
        "硬截止遗留运行必须由下一轮恢复为 interrupted",
    )?;
    recovery_lock.release().await?;
    Ok(())
}

async fn verify_non_lock_hard_deadline(database: &TestDatabase) -> TestResult {
    let task = MaintenanceTask::ExpiredSessions;
    install_slow_session_delete(database).await?;
    let runner = database.runner();
    let (sender, shutdown) = watch::channel(ShutdownSignal::Running);
    let future = runner.run_once(task, RunTrigger::Manual, shutdown);
    tokio::pin!(future);
    tokio::select! {
        biased;
        _ = &mut future => return Err(boxed("慢业务 SQL 不得在观察到执行前结束")),
        observed = wait_for_runner_sleep(database) => observed?,
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    sender
        .send(ShutdownSignal::Requested(deadline))
        .map_err(|_| boxed("无法发送慢业务 SQL 的硬截止信号"))?;
    let result = tokio::time::timeout_at(deadline + Duration::from_secs(2), &mut future)
        .await
        .map_err(|_| boxed("慢业务 SQL 超过硬截止及测试容差仍未退出"))?;
    require(
        result.is_err(),
        "旧 backend 未退出时不得伪造 cancelled 终态",
    )?;
    require(
        database.running_count(task).await? == 1,
        "硬截止后必须保留一个可恢复的 running 事实",
    )?;
    require(
        Service::new(database.observer.clone())
            .try_lock(task)
            .await?
            .is_none(),
        "旧 backend 仍执行时 advisory lock 必须阻止下一执行者",
    )?;

    let mut recovery_lock = eventually_acquire_lock(database, task).await?;
    let recovered = Service::new(database.observer.clone())
        .recover_interrupted_on(recovery_lock.connection(), task)
        .await?;
    require(recovered == 1, "旧会话退出后下一持锁者必须恢复运行记录")?;
    let record = Service::new(database.observer.clone())
        .run_on(
            recovery_lock.connection(),
            database.only_run_id(task).await?,
        )
        .await?;
    require(
        record.outcome == RunOutcome::Interrupted && record.error == Some(ErrorCode::Interrupted),
        "跨过硬截止的未知 SQL 只能恢复为 interrupted",
    )?;
    recovery_lock.release().await?;
    Ok(())
}

async fn install_slow_session_delete(database: &TestDatabase) -> TestResult {
    let account_id = Uuid::now_v7();
    sqlx::query("INSERT INTO accounts (id, email, password_hash) VALUES ($1, $2, 'test-hash')")
        .bind(account_id)
        .bind(format!("maintenance-{}@example.com", Uuid::now_v7()))
        .execute(&database.observer)
        .await?;
    sqlx::query(
        "INSERT INTO sessions (id, account_id, token_hash, expires_at) VALUES ($1, $2, $3, '2000-01-01T00:00:00Z')",
    )
    .bind(Uuid::now_v7())
    .bind(account_id)
    .bind(Uuid::now_v7().as_bytes().to_vec())
    .execute(&database.observer)
    .await?;
    sqlx::raw_sql(
        r#"
        CREATE FUNCTION maintenance_runner_slow_delete() RETURNS trigger
        LANGUAGE plpgsql AS $$
        BEGIN
            PERFORM pg_sleep(4);
            RETURN OLD;
        END;
        $$;
        CREATE TRIGGER maintenance_runner_slow_delete
        BEFORE DELETE ON sessions
        FOR EACH ROW EXECUTE FUNCTION maintenance_runner_slow_delete();
        "#,
    )
    .execute(&database.observer)
    .await?;
    Ok(())
}

async fn wait_for_runner_lock(database: &TestDatabase) -> TestResult {
    let observed = tokio::time::timeout(OBSERVE_TIMEOUT, async {
        loop {
            let waiting = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE application_name = $1 AND wait_event_type = 'Lock')",
            )
            .bind(&database.runner_application_name)
            .fetch_one(&database.observer)
            .await?;
            if waiting {
                return Ok::<(), sqlx::Error>(());
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| boxed("未观察到真实 runner backend 进入 PostgreSQL Lock 等待"))?;
    observed.map_err(Into::into)
}

async fn wait_for_runner_sleep(database: &TestDatabase) -> TestResult {
    let observed = tokio::time::timeout(OBSERVE_TIMEOUT, async {
        loop {
            let waiting = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE application_name = $1 AND wait_event = 'PgSleep')",
            )
            .bind(&database.runner_application_name)
            .fetch_one(&database.observer)
            .await?;
            if waiting {
                return Ok::<(), sqlx::Error>(());
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| boxed("未观察到真实 runner backend 进入非锁等待"))?;
    observed.map_err(Into::into)
}

async fn acquire_lock(runner: &Runner, task: MaintenanceTask) -> TestResult<AdvisoryLock> {
    runner
        .services
        .maintenance
        .try_lock(task)
        .await?
        .ok_or_else(|| boxed("本轮随机 schema 的首个持有者必须取得任务锁"))
}

async fn eventually_acquire_lock(
    database: &TestDatabase,
    task: MaintenanceTask,
) -> TestResult<AdvisoryLock> {
    let service = Service::new(database.observer.clone());
    tokio::time::timeout(OBSERVE_TIMEOUT, async {
        loop {
            if let Some(lock) = service.try_lock(task).await? {
                return Ok::<AdvisoryLock, cloud_domain::AppError>(lock);
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| boxed("被淘汰 backend 未及时释放 advisory lock"))?
    .map_err(Into::into)
}

async fn assert_lock_available(database: &TestDatabase, task: MaintenanceTask) -> TestResult {
    let lock = eventually_acquire_lock(database, task).await?;
    lock.release().await?;
    Ok(())
}

fn run_start(task: MaintenanceTask) -> RunStart {
    RunStart {
        run_id: Uuid::now_v7(),
        task,
        trigger: RunTrigger::Manual,
        instance_id: Uuid::now_v7(),
        cutoff_at: None,
        active_cutoff_at: None,
    }
}

fn cancelled_completion(start: &RunStart) -> RunCompletion {
    RunCompletion {
        run_id: start.run_id,
        task: start.task,
        outcome: RunOutcome::Cancelled,
        observation: None,
        error: Some(ErrorCode::Cancelled),
        report: TaskExecutionReport::default(),
    }
}
