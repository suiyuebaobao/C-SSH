//! 只把一次解析后确认可公开路由或测试环境精确放行的地址交给拨号器。

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use anyhow::{Result as ConfigResult, bail};
use thiserror::Error;
use tokio::{net::TcpStream, task::JoinSet};

const MAX_UNIQUE_RESOLVED_ADDRESSES: usize = 32;
const MAX_VERIFIED_DIAL_ATTEMPTS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Environment {
    Test,
    Staging,
    Production,
}

impl Environment {
    fn parse(value: &str) -> ConfigResult<Self> {
        match value {
            "test" => Ok(Self::Test),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => bail!("CLOUD_PROXY_GATEWAY_ENVIRONMENT 必须是 test、staging 或 production"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct TargetPolicy {
    test_allowlist: Arc<HashSet<SocketAddr>>,
}

#[derive(Debug, Error)]
pub(crate) enum TargetConnectError {
    #[error("target resolution failed")]
    Resolution,
    #[error("target address rejected")]
    Rejected,
    #[error("target unavailable")]
    Unavailable,
}

impl TargetPolicy {
    pub(crate) fn from_configuration(
        environment: &str,
        raw_test_allowlist: Option<&str>,
    ) -> ConfigResult<Self> {
        let environment = Environment::parse(environment)?;
        if raw_test_allowlist.is_some() && environment != Environment::Test {
            bail!("CLOUD_PROXY_GATEWAY_TEST_TARGET_ALLOWLIST 只能在 environment=test 时启用");
        }
        let test_allowlist =
            raw_test_allowlist.map_or_else(|| Ok(HashSet::new()), parse_test_allowlist)?;
        Ok(Self {
            test_allowlist: Arc::new(test_allowlist),
        })
    }

    pub(crate) async fn connect(
        &self,
        host: &str,
        port: u16,
    ) -> Result<TcpStream, TargetConnectError> {
        let resolved = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| TargetConnectError::Resolution)?;
        let verified = self.filter_resolved_addresses(resolved)?;
        if verified.is_empty() {
            return Err(TargetConnectError::Rejected);
        }

        // 只连接冻结后的少量 SocketAddr，绝不把主机名再次交给解析器。
        // 两个上限在创建任务前已强制，单条 CONNECT 无法放大为无界出站 socket。
        let mut attempts = JoinSet::new();
        for address in verified {
            attempts.spawn(async move { TcpStream::connect(address).await });
        }
        while let Some(result) = attempts.join_next().await {
            if let Ok(Ok(stream)) = result {
                attempts.abort_all();
                return Ok(stream);
            }
        }
        Err(TargetConnectError::Unavailable)
    }

    fn filter_resolved_addresses(
        &self,
        addresses: impl IntoIterator<Item = SocketAddr>,
    ) -> Result<Vec<SocketAddr>, TargetConnectError> {
        let mut seen = HashSet::new();
        let mut verified = Vec::new();
        for address in addresses {
            if !seen.insert(address) {
                continue;
            }
            if seen.len() > MAX_UNIQUE_RESOLVED_ADDRESSES {
                return Err(TargetConnectError::Rejected);
            }
            if !self.allows(address) {
                continue;
            }
            if verified.len() >= MAX_VERIFIED_DIAL_ATTEMPTS {
                return Err(TargetConnectError::Rejected);
            }
            verified.push(address);
        }
        Ok(verified)
    }

    fn allows(&self, address: SocketAddr) -> bool {
        is_public_ip(address.ip()) || self.test_allowlist.contains(&address)
    }
}

fn parse_test_allowlist(value: &str) -> ConfigResult<HashSet<SocketAddr>> {
    if value.is_empty() || value.trim() != value {
        bail!("CLOUD_PROXY_GATEWAY_TEST_TARGET_ALLOWLIST 不得为空或带首尾空白");
    }
    let mut allowlist = HashSet::new();
    for entry in value.split(',') {
        if entry.is_empty() || entry.trim() != entry {
            bail!("测试目标白名单必须是无空白的逗号分隔精确 IP:port");
        }
        let address = entry
            .parse::<SocketAddr>()
            .map_err(|_| anyhow::anyhow!("测试目标白名单条目必须是精确字面量 IP:port"))?;
        if !allowlist.insert(address) {
            bail!("测试目标白名单条目不得重复");
        }
    }
    Ok(allowlist)
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    if address == Ipv4Addr::new(168, 63, 129, 16) {
        // Azure WireServer 是来宾网络中的平台虚拟地址，不是公网 SSH 目标。
        return false;
    }
    let denied = [
        (Ipv4Addr::new(0, 0, 0, 0), 8),
        (Ipv4Addr::new(10, 0, 0, 0), 8),
        (Ipv4Addr::new(100, 64, 0, 0), 10),
        (Ipv4Addr::new(127, 0, 0, 0), 8),
        (Ipv4Addr::new(169, 254, 0, 0), 16),
        (Ipv4Addr::new(172, 16, 0, 0), 12),
        (Ipv4Addr::new(192, 0, 0, 0), 24),
        (Ipv4Addr::new(192, 0, 2, 0), 24),
        (Ipv4Addr::new(192, 88, 99, 0), 24),
        (Ipv4Addr::new(192, 168, 0, 0), 16),
        (Ipv4Addr::new(198, 18, 0, 0), 15),
        (Ipv4Addr::new(198, 51, 100, 0), 24),
        (Ipv4Addr::new(203, 0, 113, 0), 24),
        (Ipv4Addr::new(224, 0, 0, 0), 4),
        (Ipv4Addr::new(240, 0, 0, 0), 4),
    ];
    !denied
        .into_iter()
        .any(|(network, prefix)| ipv4_in_network(address, network, prefix))
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if !ipv6_in_network(address, Ipv6Addr::new(0x2000, 0, 0, 0, 0, 0, 0, 0), 3) {
        return false;
    }
    let denied = [
        // IETF protocol-assignment block is not a general-purpose public host range.
        (Ipv6Addr::new(0x2001, 0, 0, 0, 0, 0, 0, 0), 23),
        (Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 32),
        (Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 0), 16),
        (Ipv6Addr::new(0x3fff, 0, 0, 0, 0, 0, 0, 0), 20),
    ];
    !denied
        .into_iter()
        .any(|(network, prefix)| ipv6_in_network(address, network, prefix))
}

fn ipv4_in_network(address: Ipv4Addr, network: Ipv4Addr, prefix: u32) -> bool {
    let mask = u32::MAX << (32 - prefix);
    u32::from(address) & mask == u32::from(network) & mask
}

fn ipv6_in_network(address: Ipv6Addr, network: Ipv6Addr, prefix: u32) -> bool {
    let mask = u128::MAX << (128 - prefix);
    u128::from_be_bytes(address.octets()) & mask == u128::from_be_bytes(network.octets()) & mask
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::{TargetPolicy, is_public_ip};

    #[test]
    fn public_filter_rejects_special_and_documentation_ranges() {
        for rejected in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "168.63.129.16",
            "169.254.169.254",
            "172.16.0.1",
            "192.0.2.1",
            "192.168.0.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::ffff:127.0.0.1",
            "fc00::1",
            "fe80::1",
            "ff02::1",
            "2001:100::1",
            "2001:db8::1",
            "3fff::1",
        ] {
            let address = rejected.parse::<IpAddr>().expect("测试地址应有效");
            assert!(!is_public_ip(address), "特殊地址不得放行：{rejected}");
        }
        for allowed in ["93.184.216.34", "2606:4700:4700::1111"] {
            let address = allowed.parse::<IpAddr>().expect("测试地址应有效");
            assert!(is_public_ip(address), "公网地址应放行：{allowed}");
        }
    }

    #[test]
    fn exact_test_allowlist_is_port_bound_and_filters_each_resolution() {
        let policy =
            TargetPolicy::from_configuration("test", Some("192.0.2.10:22,[2001:db8::10]:2222"))
                .expect("测试环境精确白名单应有效");
        let verified = policy
            .filter_resolved_addresses([
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)), 22),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)), 23),
                "[2001:db8::10]:2222".parse().expect("IPv6 地址应有效"),
            ])
            .expect("有界解析结果应有效");
        assert_eq!(verified.len(), 2);
        assert!(verified.iter().all(|address| address.port() != 23));
    }

    #[test]
    fn excessive_dns_results_are_rejected_before_dial_tasks_exist() {
        let policy =
            TargetPolicy::from_configuration("production", None).expect("生产目标策略应有效");
        let too_many_public =
            (1..=9).map(|last| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, last)), 22));
        assert!(policy.filter_resolved_addresses(too_many_public).is_err());

        let too_many_unique =
            (1..=33).map(|last| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, last)), 22));
        assert!(policy.filter_resolved_addresses(too_many_unique).is_err());
    }

    #[test]
    fn non_test_environment_rejects_test_allowlist() {
        for environment in ["staging", "production"] {
            assert!(TargetPolicy::from_configuration(environment, Some("192.0.2.10:22")).is_err());
        }
    }
}
