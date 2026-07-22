//! Creation Cloud 单进程服务入口，仅负责配置、依赖装配和优雅退出。
//! 业务逻辑必须留在各自模块，禁止在此实现具体用例。

mod admin_overview;
mod app;
mod command;
mod http_trace;
mod maintenance;
mod maintenance_cli;
mod request_id;
mod runtime;
mod services;
mod shutdown;

use anyhow::Result;
use cloud_config::CloudConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let command = command::from_env()?;
    let config = CloudConfig::from_env()?;
    let pool = cloud_store::connect(&config.database_url).await?;
    cloud_store::migrate(&pool).await?;
    match command {
        command::Command::PromoteAdmin(email) => {
            cloud_admin::promote_registered_admin(&pool, &email).await?;
            println!("管理员角色已更新");
            Ok(())
        }
        command::Command::SetAdminLogin {
            registered_email,
            admin_login_name,
        } => {
            cloud_admin::set_registered_admin_login(&pool, &registered_email, &admin_login_name)
                .await?;
            println!("管理员登录名已更新");
            Ok(())
        }
        command::Command::Serve => {
            let services = services::AppServices::new(pool, &config);
            runtime::serve(services, config).await
        }
        command::Command::MaintenanceRun(task) => {
            let maintenance_config = config.maintenance.clone();
            let services = services::AppServices::new(pool, &config);
            maintenance_cli::run(services, maintenance_config, task).await
        }
        command::Command::MaintenanceStatus(task) => {
            let services = services::AppServices::new(pool, &config);
            maintenance_cli::status(services, task).await
        }
    }
}

fn init_tracing() {
    let filter = std::env::var("CLOUD_LOG")
        .unwrap_or_else(|_| "creation_cloud=info,tower_http=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();
}
