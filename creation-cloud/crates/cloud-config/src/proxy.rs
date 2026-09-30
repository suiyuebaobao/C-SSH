//! 解析官方代理控制面的可选密钥环与独立内网监听配置。

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use uuid::Uuid;

const DEFAULT_CREDENTIAL_TTL_SECONDS: u64 = 600;
const MAX_CREDENTIAL_TTL_SECONDS: u64 = 900;
const MAX_KEY_COUNT: usize = 4;

#[derive(Clone)]
pub struct ProxyConfig {
    credential_keys: Arc<BTreeMap<i32, Arc<[u8]>>>,
    active_key_version: Option<i32>,
    pub credential_ttl: Duration,
    pub internal_bind_addr: Option<SocketAddr>,
    pub rdp_node_ids: BTreeSet<Uuid>,
}

impl ProxyConfig {
    pub(crate) fn from_env() -> Result<Self> {
        let raw_keys = super::read_optional("CLOUD_PROXY_CREDENTIAL_KEYS")?;
        let raw_active_version = super::read_optional("CLOUD_PROXY_ACTIVE_KEY_VERSION")?;
        let raw_bind_addr = super::read_optional("CLOUD_PROXY_INTERNAL_BIND_ADDR")?;
        let gateway_verified = super::read_optional("CLOUD_PROXY_FAIL_CLOSED_GATEWAY_VERIFIED")?;
        let rdp_node_ids =
            parse_rdp_nodes(super::read_optional("CLOUD_PROXY_RDP_NODE_IDS")?.as_deref())?;
        let configured = raw_keys.is_some()
            || raw_active_version.is_some()
            || raw_bind_addr.is_some()
            || gateway_verified.is_some()
            || !rdp_node_ids.is_empty();
        if !configured {
            return Ok(Self::default());
        }
        if gateway_verified.as_deref() != Some("true") {
            bail!(
                "官方代理保持关闭：必须先验证独立失败关闭网关，再显式设置 CLOUD_PROXY_FAIL_CLOSED_GATEWAY_VERIFIED=true"
            );
        }
        let keys = parse_keys(
            raw_keys
                .as_deref()
                .context("启用官方代理时缺少 CLOUD_PROXY_CREDENTIAL_KEYS")?,
        )?;
        let active_key_version = raw_active_version
            .as_deref()
            .context("启用官方代理时缺少 CLOUD_PROXY_ACTIVE_KEY_VERSION")?
            .parse::<i32>()
            .context("CLOUD_PROXY_ACTIVE_KEY_VERSION 必须是正整数")?;
        if active_key_version <= 0 || !keys.contains_key(&active_key_version) {
            bail!("CLOUD_PROXY_ACTIVE_KEY_VERSION 必须引用已配置的正整数密钥版本");
        }
        let internal_bind_addr = raw_bind_addr
            .as_deref()
            .context("启用官方代理时缺少 CLOUD_PROXY_INTERNAL_BIND_ADDR")?
            .parse::<SocketAddr>()
            .context("CLOUD_PROXY_INTERNAL_BIND_ADDR 不是合法监听地址")?;
        if !internal_bind_addr.ip().is_loopback() {
            bail!("CLOUD_PROXY_INTERNAL_BIND_ADDR 必须是回环地址，禁止公开监听内部鉴权");
        }
        let credential_ttl = parse_ttl()?;
        Ok(Self {
            credential_keys: Arc::new(keys),
            active_key_version: Some(active_key_version),
            credential_ttl,
            internal_bind_addr: Some(internal_bind_addr),
            rdp_node_ids,
        })
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.active_key_version.is_some() && self.internal_bind_addr.is_some()
    }

    #[must_use]
    pub fn active_key(&self) -> Option<(i32, Arc<[u8]>)> {
        let version = self.active_key_version?;
        self.credential_keys
            .get(&version)
            .cloned()
            .map(|key| (version, key))
    }

    #[must_use]
    pub fn key(&self, version: i32) -> Option<Arc<[u8]>> {
        self.credential_keys.get(&version).cloned()
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            credential_keys: Arc::new(BTreeMap::new()),
            active_key_version: None,
            credential_ttl: Duration::from_secs(DEFAULT_CREDENTIAL_TTL_SECONDS),
            internal_bind_addr: None,
            rdp_node_ids: BTreeSet::new(),
        }
    }
}

impl fmt::Debug for ProxyConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProxyConfig")
            .field("enabled", &self.is_enabled())
            .field("key_count", &self.credential_keys.len())
            .field("active_key_version", &self.active_key_version)
            .field("credential_ttl", &self.credential_ttl)
            .field("internal_bind_addr", &self.internal_bind_addr)
            .field("rdp_node_count", &self.rdp_node_ids.len())
            .finish()
    }
}

fn parse_rdp_nodes(value: Option<&str>) -> Result<BTreeSet<Uuid>> {
    let mut nodes = BTreeSet::new();
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Ok(nodes);
    };
    for entry in value.split(',') {
        let id =
            Uuid::parse_str(entry).context("CLOUD_PROXY_RDP_NODE_IDS 必须是逗号分隔的节点 UUID")?;
        if id.is_nil() || !nodes.insert(id) {
            bail!("CLOUD_PROXY_RDP_NODE_IDS 不得包含零 UUID 或重复节点");
        }
    }
    Ok(nodes)
}

fn parse_keys(value: &str) -> Result<BTreeMap<i32, Arc<[u8]>>> {
    let mut keys = BTreeMap::new();
    for entry in value.split(',') {
        let (version, encoded) = entry
            .split_once(':')
            .context("CLOUD_PROXY_CREDENTIAL_KEYS 条目必须为 version:64-hex")?;
        let version = version
            .parse::<i32>()
            .context("官方代理密钥版本必须是正整数")?;
        if version <= 0 || encoded.len() != 64 {
            bail!("官方代理密钥必须使用正整数版本和 32 字节十六进制值");
        }
        let decoded = hex::decode(encoded).context("官方代理密钥必须是十六进制")?;
        if decoded.len() != 32 || keys.insert(version, Arc::from(decoded)).is_some() {
            bail!("官方代理密钥版本不得重复，且每把密钥必须恰好 32 字节");
        }
    }
    if keys.is_empty() || keys.len() > MAX_KEY_COUNT {
        bail!("官方代理密钥环必须包含 1 到 {MAX_KEY_COUNT} 把密钥");
    }
    Ok(keys)
}

fn parse_ttl() -> Result<Duration> {
    let seconds = super::read_optional("CLOUD_PROXY_CREDENTIAL_TTL_SECONDS")?
        .unwrap_or_else(|| DEFAULT_CREDENTIAL_TTL_SECONDS.to_string())
        .parse::<u64>()
        .context("CLOUD_PROXY_CREDENTIAL_TTL_SECONDS 必须是整数")?;
    if !(60..=MAX_CREDENTIAL_TTL_SECONDS).contains(&seconds) {
        bail!("CLOUD_PROXY_CREDENTIAL_TTL_SECONDS 必须在 60 到 900 之间");
    }
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::parse_rdp_nodes;

    #[test]
    fn rdp_nodes_require_explicit_valid_unique_ids() {
        assert!(parse_rdp_nodes(None).expect("默认关闭 RDP").is_empty());
        assert!(
            parse_rdp_nodes(Some(""))
                .expect("空列表关闭 RDP")
                .is_empty()
        );
        let id = "00000000-0000-0000-0000-000000000001";
        assert_eq!(
            parse_rdp_nodes(Some(id)).expect("节点 UUID 应有效").len(),
            1
        );
        for invalid in [
            "unknown",
            "00000000-0000-0000-0000-000000000000",
            &format!("{id},{id}"),
            &format!("{id},"),
        ] {
            assert!(parse_rdp_nodes(Some(invalid)).is_err());
        }
    }
}
