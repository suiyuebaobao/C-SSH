//! 解析公开根地址，并对生产环境施加不可回退的外部地址边界。

use anyhow::{Context, Result, bail};
use url::{Host, Url};

const DEFAULT_PUBLIC_BASE_URL: &str = "http://127.0.0.1:8088";

pub(super) fn resolve(environment: &str, configured_value: Option<&str>) -> Result<Url> {
    if environment == "production" && configured_value.is_none_or(|value| value.trim().is_empty()) {
        bail!("production 环境必须显式提供 CLOUD_PUBLIC_BASE_URL");
    }
    let value = match configured_value {
        Some(value) => value,
        None => DEFAULT_PUBLIC_BASE_URL,
    };
    let url = parse_root(value)?;
    if environment == "production" {
        validate_production(&url)?;
    }
    Ok(url)
}

fn parse_root(value: &str) -> Result<Url> {
    let mut url = Url::parse(value).context("CLOUD_PUBLIC_BASE_URL 不是合法 URL")?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("CLOUD_PUBLIC_BASE_URL 只能使用 http 或 https 协议");
    }
    if url.host().is_none() {
        bail!("CLOUD_PUBLIC_BASE_URL 必须包含 host");
    }
    if !url.username().is_empty() || url.password().is_some() {
        bail!("CLOUD_PUBLIC_BASE_URL 禁止包含用户名或密码");
    }
    if url.query().is_some() {
        bail!("CLOUD_PUBLIC_BASE_URL 禁止包含查询参数");
    }
    if url.fragment().is_some() {
        bail!("CLOUD_PUBLIC_BASE_URL 禁止包含片段");
    }
    if !has_only_root_path(value) || !matches!(url.path(), "" | "/") {
        bail!("CLOUD_PUBLIC_BASE_URL 路径只能为空或 /");
    }
    url.set_path("/");
    Ok(url)
}

fn validate_production(url: &Url) -> Result<()> {
    if url.scheme() != "https" {
        bail!("production 环境的 CLOUD_PUBLIC_BASE_URL 必须使用 https");
    }

    let Some(host) = url.host() else {
        bail!("CLOUD_PUBLIC_BASE_URL 必须包含 host");
    };
    match host {
        Host::Domain(domain) if is_localhost_name(domain) => {
            bail!("production 环境的 CLOUD_PUBLIC_BASE_URL 禁止使用 localhost 域名");
        }
        Host::Domain(_) => {}
        Host::Ipv4(_) | Host::Ipv6(_) => {
            bail!("production 环境的 CLOUD_PUBLIC_BASE_URL 必须使用域名，禁止使用 IP literal");
        }
    }
    Ok(())
}

fn is_localhost_name(domain: &str) -> bool {
    let domain = domain.trim_end_matches('.');
    domain.eq_ignore_ascii_case("localhost")
        || domain
            .to_ascii_lowercase()
            .strip_suffix(".localhost")
            .is_some_and(|prefix| !prefix.is_empty())
}

fn has_only_root_path(value: &str) -> bool {
    let Some((_, after_scheme)) = value.split_once(':') else {
        return false;
    };
    let Some(after_authority) = after_scheme.strip_prefix("//") else {
        return false;
    };
    match after_authority.find(['/', '\\']) {
        Some(path_start) => &after_authority[path_start..] == "/",
        None => true,
    }
}
