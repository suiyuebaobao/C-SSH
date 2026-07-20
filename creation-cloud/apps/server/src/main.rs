//! Creation Cloud 单进程服务入口，仅负责配置、依赖装配和优雅退出。
//! 业务逻辑必须留在各自模块，禁止在此实现具体用例。

mod admin_overview;
mod app;
mod command;
mod http_trace;
mod request_id;
mod shutdown;

use anyhow::Result;
use cloud_config::CloudConfig;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let command = command::from_env()?;
    let config = CloudConfig::from_env()?;
    tokio::fs::create_dir_all(&config.download_root).await?;
    tokio::fs::create_dir_all(&config.site_media_root).await?;
    let pool = cloud_store::connect(&config.database_url).await?;
    cloud_store::migrate(&pool).await?;
    if let command::Command::PromoteAdmin(email) = command {
        cloud_admin::promote_registered_admin(&pool, &email).await?;
        println!("管理员角色已更新");
        return Ok(());
    }
    let router = app::build(pool, config.clone());
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    info!(address = %config.bind_addr, "Creation Cloud 已启动");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown::signal())
        .await?;
    Ok(())
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
