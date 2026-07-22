//! 提供严格 JSON 的单次维护运行与只读状态命令，并复用 HTTP 进程同一套 runner。

use std::io::{self, Write};

use anyhow::Result;
use cloud_maintenance::{MaintenanceTask, RunTrigger};
use serde::Serialize;
use tokio::{sync::watch, time::Instant};

use crate::{
    maintenance::{Runner, ShutdownSignal},
    services::AppServices,
    shutdown,
};

pub async fn run(
    services: AppServices,
    config: cloud_config::MaintenanceConfig,
    task: MaintenanceTask,
) -> Result<()> {
    let shutdown_timeout = config.shutdown_timeout;
    let runner = Runner::new(services, config);
    let (shutdown_sender, shutdown_receiver) = watch::channel(ShutdownSignal::Running);
    let execution = runner.run_once(task, RunTrigger::Manual, shutdown_receiver);
    tokio::pin!(execution);
    let record = tokio::select! {
        result = &mut execution => result?,
        () = shutdown::signal() => {
            let deadline = Instant::now() + shutdown_timeout;
            let _ = shutdown_sender.send(ShutdownSignal::Requested(deadline));
            execution.await?
        }
    };
    write_json(&record)
}

pub async fn status(services: AppServices, task: Option<MaintenanceTask>) -> Result<()> {
    match task {
        Some(task) => {
            let status = services.maintenance.status(task).await?;
            write_json(&status)
        }
        None => {
            let statuses = services.maintenance.statuses().await?;
            write_json(&statuses)
        }
    }
}

fn write_json<T: Serialize>(value: &T) -> Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, value)?;
    output.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
#[path = "maintenance_cli_tests.rs"]
mod tests;
