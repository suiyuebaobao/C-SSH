//! 从环境变量加载网关配置，并拒绝非回环的控制面鉴权地址。

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use url::{Host, Url};
use zeroize::Zeroizing;

use crate::target::TargetPolicy;

const AUTHORIZATION_PATH: &str = "/internal/v1/proxy/authorize";
const CLOSE_PATH: &str = "/internal/v1/proxy/close";

#[derive(Clone)]
pub(crate) struct AuthorizationEndpoint {
    pub address: SocketAddr,
    pub host_header: String,
    pub authorization_path: &'static str,
    pub close_path: &'static str,
}

impl AuthorizationEndpoint {
    fn parse(value: &str) -> Result<Self> {
        let url =
            Url::parse(value).context("CLOUD_PROXY_GATEWAY_AUTHORIZATION_URL 不是合法 URL")?;
        if url.scheme() != "http"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != AUTHORIZATION_PATH
            || url.query().is_some()
            || url.fragment().is_some()
        {
            bail!("网关鉴权 URL 必须是无用户信息、查询和片段的回环 HTTP 固定路径");
        }
        let ip = match url.host() {
            Some(Host::Ipv4(ip)) => IpAddr::V4(ip),
            Some(Host::Ipv6(ip)) => IpAddr::V6(ip),
            _ => bail!("网关鉴权 URL 必须使用字面量回环 IP，禁止 DNS 和公网地址"),
        };
        if !ip.is_loopback() {
            bail!("网关鉴权 URL 必须使用回环 IP");
        }
        let port = url
            .port_or_known_default()
            .context("网关鉴权 URL 缺少端口")?;
        let host_header = match ip {
            IpAddr::V4(value) => format!("{value}:{port}"),
            IpAddr::V6(value) => format!("[{value}]:{port}"),
        };
        Ok(Self {
            address: SocketAddr::new(ip, port),
            host_header,
            authorization_path: AUTHORIZATION_PATH,
            close_path: CLOSE_PATH,
        })
    }
}

#[derive(Clone)]
pub(crate) struct Config {
    pub bind_addr: SocketAddr,
    pub tls_cert_file: PathBuf,
    pub tls_key_file: PathBuf,
    pub authorization_endpoint: AuthorizationEndpoint,
    pub node_token: Arc<Zeroizing<String>>,
    pub request_timeout: Duration,
    pub authorization_timeout: Duration,
    pub target_connect_timeout: Duration,
    pub reauthorize_interval: Duration,
    pub max_connections: usize,
    pub max_connections_per_source: usize,
    pub target_policy: TargetPolicy,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bind_addr = required("CLOUD_PROXY_GATEWAY_BIND_ADDR")?
            .parse::<SocketAddr>()
            .context("CLOUD_PROXY_GATEWAY_BIND_ADDR 不是合法监听地址")?;
        let tls_cert_file = PathBuf::from(required("CLOUD_PROXY_GATEWAY_TLS_CERT_FILE")?);
        let tls_key_file = PathBuf::from(required("CLOUD_PROXY_GATEWAY_TLS_KEY_FILE")?);
        let authorization_endpoint =
            AuthorizationEndpoint::parse(&required("CLOUD_PROXY_GATEWAY_AUTHORIZATION_URL")?)?;
        let node_token = required("CLOUD_PROXY_GATEWAY_NODE_TOKEN")?;
        validate_token(&node_token)?;
        let environment = required("CLOUD_PROXY_GATEWAY_ENVIRONMENT")?;
        let raw_test_allowlist = optional("CLOUD_PROXY_GATEWAY_TEST_TARGET_ALLOWLIST")?;
        let target_policy =
            TargetPolicy::from_configuration(&environment, raw_test_allowlist.as_deref())?;
        let max_connections = integer("CLOUD_PROXY_GATEWAY_MAX_CONNECTIONS", 256, 1, 4_096)?;
        let max_connections_per_source =
            integer("CLOUD_PROXY_GATEWAY_MAX_CONNECTIONS_PER_SOURCE", 8, 1, 64)?;
        if max_connections_per_source > max_connections {
            bail!("CLOUD_PROXY_GATEWAY_MAX_CONNECTIONS_PER_SOURCE 不得超过总并发上限");
        }
        Ok(Self {
            bind_addr,
            tls_cert_file,
            tls_key_file,
            authorization_endpoint,
            node_token: Arc::new(Zeroizing::new(node_token)),
            request_timeout: duration_seconds(
                "CLOUD_PROXY_GATEWAY_REQUEST_TIMEOUT_SECONDS",
                10,
                1,
                30,
            )?,
            authorization_timeout: duration_millis(
                "CLOUD_PROXY_GATEWAY_AUTHORIZATION_TIMEOUT_MILLISECONDS",
                2_000,
                100,
                5_000,
            )?,
            target_connect_timeout: duration_seconds(
                "CLOUD_PROXY_GATEWAY_TARGET_CONNECT_TIMEOUT_SECONDS",
                10,
                1,
                30,
            )?,
            reauthorize_interval: duration_seconds(
                "CLOUD_PROXY_GATEWAY_REAUTHORIZE_INTERVAL_SECONDS",
                2,
                1,
                5,
            )?,
            max_connections,
            max_connections_per_source,
            target_policy,
        })
    }
}

fn required(name: &str) -> Result<String> {
    let value = std::env::var(name).with_context(|| format!("缺少 {name}"))?;
    if value.is_empty() || value.trim() != value {
        bail!("{name} 不得为空或带首尾空白");
    }
    Ok(value)
}

fn optional(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => {
            if value.is_empty() || value.trim() != value {
                bail!("{name} 不得为空或带首尾空白");
            }
            Ok(Some(value))
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => bail!("{name} 必须是 Unicode"),
    }
}

fn validate_token(token: &str) -> Result<()> {
    if token.len() < 32
        || token.len() > 512
        || !token.is_ascii()
        || token.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        bail!("CLOUD_PROXY_GATEWAY_NODE_TOKEN 必须是 32 到 512 字节无空白 ASCII");
    }
    Ok(())
}

fn duration_seconds(name: &str, default: u64, min: u64, max: u64) -> Result<Duration> {
    Ok(Duration::from_secs(integer(name, default, min, max)?))
}

fn duration_millis(name: &str, default: u64, min: u64, max: u64) -> Result<Duration> {
    Ok(Duration::from_millis(integer(name, default, min, max)?))
}

fn integer<T>(name: &str, default: T, min: T, max: T) -> Result<T>
where
    T: Copy + Ord + std::str::FromStr + std::fmt::Display,
    <T as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let value = std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .parse::<T>()
        .with_context(|| format!("{name} 必须是整数"))?;
    if value < min || value > max {
        bail!("{name} 必须在 {min} 到 {max} 之间");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::AuthorizationEndpoint;

    #[test]
    fn authorization_endpoint_requires_literal_loopback_and_fixed_path() {
        let endpoint =
            AuthorizationEndpoint::parse("http://127.0.0.1:3101/internal/v1/proxy/authorize")
                .expect("回环固定路径应有效");
        assert!(endpoint.address.ip().is_loopback());
        for rejected in [
            "https://127.0.0.1:3101/internal/v1/proxy/authorize",
            "http://localhost:3101/internal/v1/proxy/authorize",
            "http://192.0.2.1:3101/internal/v1/proxy/authorize",
            "http://127.0.0.1:3101/api/v1/proxy/authorize",
            "http://127.0.0.1:3101/internal/v1/proxy/authorize?x=1",
        ] {
            assert!(AuthorizationEndpoint::parse(rejected).is_err());
        }
    }
}
