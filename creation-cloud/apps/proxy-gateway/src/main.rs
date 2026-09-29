//! 独立、失败关闭的官方代理边缘入口。
//!
//! 该进程只接受 TLS 上的 HTTP CONNECT。它不会在鉴权失败时回退到其它代理或直连路径。

mod authorize;
mod config;
mod connect_request;
mod gateway;
mod rdp_negotiation;
mod source_limit;
mod ssh_banner;
mod target;
mod tls;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let config = config::Config::from_env()?;
    let tls_acceptor = tls::acceptor(&config.tls_cert_file, &config.tls_key_file)?;
    gateway::run(config, tls_acceptor).await
}

fn init_tracing() {
    let filter = std::env::var("CLOUD_PROXY_GATEWAY_LOG")
        .unwrap_or_else(|_| "creation_cloud_proxy_gateway=info".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();
}
