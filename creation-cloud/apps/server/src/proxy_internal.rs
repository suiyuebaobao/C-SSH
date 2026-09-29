//! 只在回环地址启动官方代理内部鉴权，绝不合并进公网应用 Router。

use anyhow::{Context, Result};
use tokio::{sync::watch, task::JoinHandle};

pub async fn start(
    service: cloud_proxy::Service,
    config: &cloud_config::ProxyConfig,
    shutdown_receiver: watch::Receiver<bool>,
) -> Result<Option<JoinHandle<std::io::Result<()>>>> {
    if !service.is_enabled() {
        return Ok(None);
    }
    let bind_addr = config
        .internal_bind_addr
        .context("官方代理已启用但缺少内部监听地址")?;
    if !bind_addr.ip().is_loopback() {
        anyhow::bail!("官方代理内部鉴权只能监听回环地址");
    }
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .context("官方代理内部鉴权监听失败")?;
    let router = cloud_proxy::internal_router(service);
    Ok(Some(tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(crate::shutdown::wait(shutdown_receiver))
            .await
    })))
}
