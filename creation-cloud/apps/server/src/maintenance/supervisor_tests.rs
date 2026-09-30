//! 验证 supervisor 的重试上界和最早绝对关停截止时间语义。

use super::*;

#[test]
fn skipped_or_infrastructure_failure_retries_within_one_minute() {
    let daily = Duration::from_secs(24 * 60 * 60);
    assert_eq!(
        next_wait(daily, Some(RunOutcome::SkippedLocked)),
        LOCK_RETRY_MAX
    );
    assert_eq!(next_wait(daily, None), LOCK_RETRY_MAX);
    assert_eq!(next_wait(daily, Some(RunOutcome::Failed)), daily);
}

#[test]
fn skipped_and_ordinary_errors_cannot_clear_an_uncertain_run() {
    let mut safety = RunSafety::default();
    safety.observe(failed_run_evidence(true));
    safety.observe(settled_outcome_evidence(RunOutcome::SkippedLocked));
    safety.observe(failed_run_evidence(false));
    safety.observe(settled_outcome_evidence(RunOutcome::Running));
    assert!(safety.pending_uncertain_run);
    assert!(safety.shutdown_result().is_err());
}

#[test]
fn persisted_terminal_outcome_clears_an_uncertain_run() {
    let mut safety = RunSafety::default();
    safety.observe(failed_run_evidence(true));
    safety.observe(settled_outcome_evidence(RunOutcome::Failed));
    assert!(!safety.pending_uncertain_run);
    assert!(safety.shutdown_result().is_ok());
}

#[tokio::test(start_paused = true)]
async fn shutdown_during_retry_sleep_propagates_an_uncertain_run() {
    let (shutdown_sender, mut shutdown) = watch::channel(ShutdownSignal::Running);
    let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        let mut safety = RunSafety::default();
        safety.observe(failed_run_evidence(true));
        ready_sender.send(()).expect("测试协调通道应保持连接");
        assert_eq!(
            wait_for_retry(Duration::from_secs(60), &mut shutdown).await,
            RetryWait::Shutdown
        );
        safety.shutdown_result()
    });
    let supervisor = Supervisor {
        tasks,
        shutdown_sender,
        unexpected_exit: false,
        unsettled_active_run: false,
    };
    ready_receiver.await.expect("任务应进入重试等待");
    let error = supervisor
        .shutdown_until(Instant::now() + Duration::from_secs(5))
        .await
        .expect_err("未收敛运行必须让 supervisor 关停失败");
    assert_eq!(
        error.to_string(),
        "维护 supervisor 关停时存在未收敛的活动运行"
    );
}

#[tokio::test]
async fn supervisor_keeps_the_earliest_absolute_shutdown_deadline() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://example:example@example.com/example")
        .expect("固定测试连接串应可创建惰性连接池");
    let config = cloud_config::CloudConfig {
        bind_addr: "127.0.0.1:8088".parse().expect("地址应有效"),
        database_url: "postgres://example:example@example.com/example".to_owned(),
        public_base_url: "http://127.0.0.1:8088".parse().expect("URL 应有效"),
        google_site_verification: None,
        baidu_site_verification: None,
        download_root: "data/downloads".into(),
        site_media_root: "data/site-media".into(),
        session_ttl: Duration::from_secs(3600),
        environment: "test".to_owned(),
        maintenance: cloud_config::MaintenanceConfig::default(),
        proxy: cloud_config::ProxyConfig::default(),
        smtp: None,
    };
    let services = crate::services::AppServices::new(pool, &config).expect("测试服务应可装配");
    let mut supervisor = Supervisor::start(Runner::new(services, config.maintenance));
    let early = Instant::now() + Duration::from_secs(5);
    supervisor.request_shutdown(early + Duration::from_secs(5));
    supervisor.request_shutdown(early);
    supervisor.request_shutdown(early + Duration::from_secs(10));
    assert_eq!(supervisor.shutdown_sender.borrow().deadline(), Some(early));
    supervisor.tasks.abort_all();
}

#[tokio::test]
async fn task_panic_is_visible_during_normal_service() {
    let mut tasks: JoinSet<Result<(), ()>> = JoinSet::new();
    tasks.spawn(async { panic!("维护测试任务 panic") });
    let mut supervisor = test_supervisor(tasks);

    supervisor.wait_for_unexpected_exit().await;
    assert!(supervisor.unexpected_exit);
    assert_eq!(
        *supervisor.shutdown_sender.borrow(),
        ShutdownSignal::Running,
        "panic 必须在发起关停前已被正常期监管发现"
    );
    let error = supervisor
        .shutdown_until(Instant::now() + Duration::from_secs(1))
        .await
        .expect_err("任务 panic 必须让 supervisor 失败");
    assert_eq!(error.to_string(), "维护 supervisor 运行期任务意外退出");
}

#[tokio::test]
async fn premature_clean_exit_is_visible_during_normal_service() {
    let mut tasks = JoinSet::new();
    tasks.spawn(async { Ok(()) });
    let mut supervisor = test_supervisor(tasks);

    supervisor.wait_for_unexpected_exit().await;
    assert!(supervisor.unexpected_exit);
    assert_eq!(
        *supervisor.shutdown_sender.borrow(),
        ShutdownSignal::Running,
        "提前退出必须在发起关停前已被正常期监管发现"
    );
    let error = supervisor
        .shutdown_until(Instant::now() + Duration::from_secs(1))
        .await
        .expect_err("提前正常退出同样必须让 supervisor 失败");
    assert_eq!(error.to_string(), "维护 supervisor 运行期任务意外退出");
}

fn test_supervisor(tasks: JoinSet<Result<(), ()>>) -> Supervisor {
    let (shutdown_sender, _shutdown) = watch::channel(ShutdownSignal::Running);
    Supervisor {
        tasks,
        shutdown_sender,
        unexpected_exit: false,
        unsettled_active_run: false,
    }
}
