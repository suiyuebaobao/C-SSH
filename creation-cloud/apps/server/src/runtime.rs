//! 统一监管 HTTP 与五项维护循环，共享一个操作系统信号和同一退出截止时间。

use anyhow::{Result, bail};
use tokio::{sync::watch, task::JoinHandle, time::Instant};
use tracing::info;

use crate::{
    app,
    maintenance::{Runner, Supervisor},
    services::AppServices,
    shutdown,
};

enum RuntimeExit {
    ShutdownSignal,
    ServerFinished(std::result::Result<std::io::Result<()>, tokio::task::JoinError>),
    SupervisorFailed,
}

pub async fn serve(services: AppServices, config: cloud_config::CloudConfig) -> Result<()> {
    tokio::fs::create_dir_all(&config.download_root).await?;
    tokio::fs::create_dir_all(&config.site_media_root).await?;
    let router = app::build(services.clone(), config.clone());
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let (shutdown_sender, shutdown_receiver) = watch::channel(false);
    let mut supervisor = Supervisor::start(Runner::new(services, config.maintenance.clone()));
    let mut server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown::wait(shutdown_receiver))
            .await
    });
    info!(address = %config.bind_addr, "Creation Cloud 已启动");

    let exit = tokio::select! {
        biased;
        () = supervisor.wait_for_unexpected_exit() => RuntimeExit::SupervisorFailed,
        result = &mut server => RuntimeExit::ServerFinished(result),
        () = shutdown::signal() => RuntimeExit::ShutdownSignal,
    };
    let deadline = Instant::now() + config.maintenance.shutdown_timeout;
    supervisor.request_shutdown(deadline);
    let _ = shutdown_sender.send(true);
    match exit {
        RuntimeExit::ServerFinished(server_result) => {
            let supervisor_result = supervisor.shutdown_until(deadline).await;
            let server_result = completed_server_result(server_result);
            supervisor_result?;
            server_result
        }
        RuntimeExit::SupervisorFailed => {
            let (server_result, supervisor_result) = tokio::join!(
                wait_for_server(&mut server, deadline),
                supervisor.shutdown_until(deadline),
            );
            // 运行期维护失败是本次退出的直接原因，必须优先传播固定错误。
            supervisor_result?;
            server_result
        }
        RuntimeExit::ShutdownSignal => {
            let (server_result, supervisor_result) = tokio::join!(
                wait_for_server(&mut server, deadline),
                supervisor.shutdown_until(deadline),
            );
            supervisor_result?;
            server_result
        }
    }
}

fn completed_server_result(
    result: std::result::Result<std::io::Result<()>, tokio::task::JoinError>,
) -> Result<()> {
    result??;
    Ok(())
}

async fn wait_for_server(
    server: &mut JoinHandle<std::io::Result<()>>,
    deadline: Instant,
) -> Result<()> {
    match tokio::time::timeout_at(deadline, &mut *server).await {
        Ok(result) => {
            result??;
            Ok(())
        }
        Err(_) => {
            server.abort();
            let _ = server.await;
            bail!("HTTP 服务未在统一退出时限内完成，已显式中止")
        }
    }
}
