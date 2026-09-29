//! 接受 TLS CONNECT，以双 admission、SSH／RDP 门禁和短 continuation lease 失败关闭隧道。

use std::{future::Future, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Semaphore, watch},
    task::JoinSet,
    time::{self, Instant, MissedTickBehavior},
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use uuid::Uuid;

use crate::{
    authorize::{Authorization, AuthorizationError, Client as AuthorizationClient},
    config::Config,
    connect_request::{self, ConnectRequest, ConnectRequestError, TargetProtocol},
    rdp_negotiation,
    source_limit::SourceLimiter,
    ssh_banner,
};

const BAD_REQUEST: &[u8] = b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nCache-Control: no-store\r\nPragma: no-cache\r\nContent-Length: 0\r\n\r\n";
const AUTH_REQUIRED: &[u8] = b"HTTP/1.1 407 Proxy Authentication Required\r\nProxy-Authenticate: Basic realm=\"C-SSH Official Proxy\"\r\nConnection: close\r\nCache-Control: no-store\r\nPragma: no-cache\r\nContent-Length: 0\r\n\r\n";
const SERVICE_UNAVAILABLE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\nCache-Control: no-store\r\nPragma: no-cache\r\nContent-Length: 0\r\n\r\n";
const BAD_GATEWAY: &[u8] = b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nCache-Control: no-store\r\nPragma: no-cache\r\nContent-Length: 0\r\n\r\n";
const CONNECTED: &[u8] = b"HTTP/1.1 200 Connection Established\r\n\r\n";

#[derive(Debug, Error)]
enum GatewayError {
    #[error("TLS handshake failed")]
    Tls,
    #[error("CONNECT request invalid")]
    InvalidRequest,
    #[error("proxy credentials invalid")]
    InvalidCredentials,
    #[error("authorization denied")]
    AuthorizationDenied,
    #[error("authorization unavailable")]
    AuthorizationUnavailable,
    #[error("target unavailable")]
    TargetUnavailable,
    #[error("target is not an SSH service")]
    SshBannerRejected,
    #[error("RDP negotiation rejected")]
    RdpNegotiationRejected,
    #[error("tunnel I/O failed")]
    TunnelIo,
    #[error("continuation lease expired")]
    LeaseExpired,
    #[error("gateway shutdown")]
    Shutdown,
}

impl GatewayError {
    fn code(&self) -> &'static str {
        match self {
            Self::Tls => "tls_rejected",
            Self::InvalidRequest => "request_rejected",
            Self::InvalidCredentials => "credentials_rejected",
            Self::AuthorizationDenied => "authorization_denied",
            Self::AuthorizationUnavailable => "authorization_unavailable",
            Self::TargetUnavailable => "target_unavailable",
            Self::SshBannerRejected => "ssh_banner_rejected",
            Self::RdpNegotiationRejected => "rdp_negotiation_rejected",
            Self::TunnelIo => "tunnel_closed_with_error",
            Self::LeaseExpired => "continuation_lease_expired",
            Self::Shutdown => "shutdown",
        }
    }
}

pub(crate) async fn run(config: Config, tls_acceptor: TlsAcceptor) -> Result<()> {
    let listener = TcpListener::bind(config.bind_addr)
        .await
        .context("无法监听官方代理网关")?;
    let authorization = AuthorizationClient::new(
        config.authorization_endpoint.clone(),
        Arc::clone(&config.node_token),
        config.authorization_timeout,
    );
    let capacity = Arc::new(Semaphore::new(config.max_connections));
    let source_capacity = SourceLimiter::new(config.max_connections_per_source);
    let (shutdown_sender, shutdown_receiver) = watch::channel(false);
    let mut tasks = JoinSet::new();
    tracing::info!("官方代理网关已启动");

    loop {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.context("无法等待网关退出信号")?;
                break;
            }
            accepted = listener.accept() => {
                let (socket, peer) = accepted.context("官方代理网关接受连接失败")?;
                let Ok(permit) = Arc::clone(&capacity).try_acquire_owned() else {
                    tracing::warn!(outcome = "capacity_rejected", "官方代理连接被拒绝");
                    drop(socket);
                    continue;
                };
                let Some(source_permit) = source_capacity.try_acquire(peer.ip()) else {
                    tracing::warn!(outcome = "source_capacity_rejected", "官方代理连接被拒绝");
                    drop(socket);
                    continue;
                };
                let acceptor = tls_acceptor.clone();
                let client = authorization.clone();
                let connection_config = config.clone();
                let connection_shutdown = shutdown_receiver.clone();
                tasks.spawn(async move {
                    let _permit = permit;
                    let _source_permit = source_permit;
                    let outcome = handle_connection(
                        socket,
                        acceptor,
                        client,
                        connection_config,
                        connection_shutdown,
                    ).await;
                    match outcome {
                        Ok(()) | Err(GatewayError::Shutdown) => {}
                        Err(error) => tracing::warn!(outcome = error.code(), "官方代理连接已关闭"),
                    }
                });
            }
            joined = tasks.join_next(), if !tasks.is_empty() => {
                if joined.is_some_and(|result| result.is_err()) {
                    tracing::error!(outcome = "connection_task_failed", "官方代理连接任务异常结束");
                }
            }
        }
    }

    let _ = shutdown_sender.send(true);
    let drained = time::timeout(Duration::from_secs(10), async {
        while tasks.join_next().await.is_some() {}
    })
    .await;
    if drained.is_err() {
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
    }
    tracing::info!("官方代理网关已停止");
    Ok(())
}

async fn handle_connection(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    authorization: AuthorizationClient,
    config: Config,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), GatewayError> {
    let mut client = time::timeout(config.request_timeout, acceptor.accept(socket))
        .await
        .map_err(|_| GatewayError::Tls)?
        .map_err(|_| GatewayError::Tls)?;
    let request = match time::timeout(
        config.request_timeout,
        connect_request::read_from(&mut client),
    )
    .await
    {
        Err(_) => {
            write_response(&mut client, BAD_REQUEST).await;
            return Err(GatewayError::InvalidRequest);
        }
        Ok(Err(ConnectRequestError::Credentials)) => {
            write_response(&mut client, AUTH_REQUIRED).await;
            return Err(GatewayError::InvalidCredentials);
        }
        Ok(Err(ConnectRequestError::Invalid)) => {
            write_response(&mut client, BAD_REQUEST).await;
            return Err(GatewayError::InvalidRequest);
        }
        Ok(Ok(request)) => request,
    };

    // UUID v4 的随机位来自系统 CSPRNG；connection_id 只作为不透明 lease 关联键。
    let connection_id = Uuid::new_v4();
    if let Err(error) =
        require_admission(&authorization, &request, connection_id, &mut client).await
    {
        let _ = authorization.close(connection_id).await;
        return Err(error);
    }

    let outcome = handle_admitted_connection(
        &mut client,
        request,
        &authorization,
        connection_id,
        &config,
        &mut shutdown,
    )
    .await;
    let _ = authorization.close(connection_id).await;
    outcome
}

async fn handle_admitted_connection(
    client: &mut TlsStream<TcpStream>,
    request: ConnectRequest,
    authorization: &AuthorizationClient,
    connection_id: Uuid,
    config: &Config,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<(), GatewayError> {
    let target = time::timeout(config.target_connect_timeout, async {
        let mut target = config
            .target_policy
            .connect(&request.host, request.port)
            .await
            .map_err(|_| GatewayError::TargetUnavailable)?;
        let banner = match request.protocol {
            TargetProtocol::Ssh => ssh_banner::read(&mut target)
                .await
                .map_err(|_| GatewayError::SshBannerRejected)?,
            TargetProtocol::Rdp => Vec::new(),
        };
        Ok::<_, GatewayError>((target, banner))
    })
    .await
    .map_err(|_| GatewayError::TargetUnavailable)?;
    let (mut target, server_banner) = match target {
        Ok(target) => target,
        Err(error @ GatewayError::TargetUnavailable)
        | Err(error @ GatewayError::SshBannerRejected) => {
            write_response(client, BAD_GATEWAY).await;
            return Err(error);
        }
        Err(error) => return Err(error),
    };

    // RDP 是客户端先发；第二次 admission 仍发生在 200 或任一隧道首字节之前。
    let allowed = require_admission(authorization, &request, connection_id, client).await?;
    let ConnectRequest {
        credentials,
        prefetched,
        protocol,
        ..
    } = request;
    drop(credentials);
    tunnel(
        client,
        &mut target,
        LeaseMonitor {
            authorization,
            connection_id,
            expires_at: allowed.lease_expires_at,
            interval: config.reauthorize_interval,
        },
        InitialData {
            server_banner: &server_banner,
            prefetched: &prefetched,
            protocol,
            handshake_timeout: config.request_timeout,
        },
        shutdown,
    )
    .await
}

async fn require_admission(
    authorization: &AuthorizationClient,
    request: &ConnectRequest,
    connection_id: Uuid,
    client: &mut TlsStream<TcpStream>,
) -> Result<Authorization, GatewayError> {
    match authorization.admit(request, connection_id).await {
        Ok(allowed) => Ok(allowed),
        Err(AuthorizationError::Denied) => {
            write_response(client, AUTH_REQUIRED).await;
            Err(GatewayError::AuthorizationDenied)
        }
        Err(AuthorizationError::Unavailable) => {
            write_response(client, SERVICE_UNAVAILABLE).await;
            Err(GatewayError::AuthorizationUnavailable)
        }
    }
}

struct LeaseMonitor<'a> {
    authorization: &'a AuthorizationClient,
    connection_id: Uuid,
    expires_at: DateTime<Utc>,
    interval: Duration,
}

struct InitialData<'a> {
    server_banner: &'a [u8],
    prefetched: &'a [u8],
    protocol: TargetProtocol,
    handshake_timeout: Duration,
}

async fn tunnel(
    client: &mut TlsStream<TcpStream>,
    target: &mut TcpStream,
    lease: LeaseMonitor<'_>,
    initial: InitialData<'_>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<(), GatewayError> {
    let data = deliver_and_transfer(client, target, initial);
    let monitor = monitor_authorization(
        lease.authorization,
        lease.connection_id,
        lease.expires_at,
        lease.interval,
    );
    run_guarded_data(data, monitor, shutdown).await
}

async fn deliver_and_transfer<C, T>(
    client: &mut C,
    target: &mut T,
    initial: InitialData<'_>,
) -> Result<(), GatewayError>
where
    C: AsyncRead + AsyncWrite + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
{
    client
        .write_all(CONNECTED)
        .await
        .map_err(|_| GatewayError::TunnelIo)?;
    match initial.protocol {
        TargetProtocol::Ssh => {
            client
                .write_all(initial.server_banner)
                .await
                .map_err(|_| GatewayError::TunnelIo)?;
            if !initial.prefetched.is_empty() {
                target
                    .write_all(initial.prefetched)
                    .await
                    .map_err(|_| GatewayError::TunnelIo)?;
            }
        }
        TargetProtocol::Rdp => {
            // lease monitor 已与该 future 一起运行，覆盖 200、双方协商等待和后续转发。
            client.flush().await.map_err(|_| GatewayError::TunnelIo)?;
            time::timeout(
                initial.handshake_timeout,
                rdp_negotiation::negotiate(client, target, initial.prefetched),
            )
            .await
            .map_err(|_| GatewayError::RdpNegotiationRejected)?
            .map_err(|_| GatewayError::RdpNegotiationRejected)?;
        }
    }
    tokio::io::copy_bidirectional(client, target)
        .await
        .map(|_| ())
        .map_err(|_| GatewayError::TunnelIo)
}

async fn run_guarded_data<D, M>(
    data: D,
    monitor: M,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<(), GatewayError>
where
    D: Future<Output = Result<(), GatewayError>>,
    M: Future<Output = GatewayError>,
{
    tokio::pin!(data);
    tokio::pin!(monitor);
    tokio::select! {
        biased;
        result = &mut monitor => Err(result),
        _ = shutdown.changed() => Err(GatewayError::Shutdown),
        result = &mut data => result,
    }
}

async fn monitor_authorization(
    authorization: &AuthorizationClient,
    connection_id: Uuid,
    mut lease_expires_at: DateTime<Utc>,
    reauthorize_interval: Duration,
) -> GatewayError {
    let mut interval =
        time::interval_at(Instant::now() + reauthorize_interval, reauthorize_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let expiry = time::sleep(duration_until(lease_expires_at));
    tokio::pin!(expiry);
    loop {
        tokio::select! {
            biased;
            _ = &mut expiry => return GatewayError::LeaseExpired,
            _ = interval.tick() => {
                let check = authorization.continue_lease(connection_id);
                tokio::pin!(check);
                let allowed = tokio::select! {
                    biased;
                    _ = &mut expiry => return GatewayError::LeaseExpired,
                    result = &mut check => match result {
                        Ok(allowed) => allowed,
                        Err(AuthorizationError::Denied) => return GatewayError::AuthorizationDenied,
                        Err(AuthorizationError::Unavailable) => {
                            return GatewayError::AuthorizationUnavailable;
                        }
                    }
                };
                lease_expires_at = allowed.lease_expires_at;
                expiry.as_mut().reset(Instant::now() + duration_until(lease_expires_at));
            }
        }
    }
}

fn duration_until(expires_at: DateTime<Utc>) -> Duration {
    expires_at
        .signed_duration_since(Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO)
}

async fn write_response<S>(stream: &mut S, response: &[u8])
where
    S: AsyncWrite + Unpin,
{
    let _ = stream.write_all(response).await;
    let _ = stream.shutdown().await;
}

#[cfg(test)]
#[path = "gateway_tests.rs"]
mod tests;
