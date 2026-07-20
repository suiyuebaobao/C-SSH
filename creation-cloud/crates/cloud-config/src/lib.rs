//! 从环境变量读取 Creation Cloud 启动配置并校验安全边界。

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use url::Url;

#[cfg(test)]
mod tests;

const DEFAULT_SITE_MEDIA_ROOT: &str = "./data/site-media";

#[derive(Clone, Debug)]
pub struct CloudConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub public_base_url: Url,
    pub google_site_verification: Option<String>,
    pub baidu_site_verification: Option<String>,
    pub download_root: PathBuf,
    pub site_media_root: PathBuf,
    pub session_ttl: Duration,
    pub environment: String,
}

impl CloudConfig {
    pub fn from_env() -> Result<Self> {
        let bind_addr = read("CLOUD_BIND_ADDR", "127.0.0.1:8088")
            .parse()
            .context("CLOUD_BIND_ADDR 不是合法监听地址")?;
        let database_url = required("CLOUD_DATABASE_URL")?;
        let public_base_url =
            parse_public_base_url(&read("CLOUD_PUBLIC_BASE_URL", "http://127.0.0.1:8088"))?;
        let google_site_verification = read_site_verification("CLOUD_GOOGLE_SITE_VERIFICATION")?;
        let baidu_site_verification = read_site_verification("CLOUD_BAIDU_SITE_VERIFICATION")?;
        let download_root = PathBuf::from(read("CLOUD_DOWNLOAD_ROOT", "./data/downloads"));
        let site_media_root = PathBuf::from(read("CLOUD_SITE_MEDIA_ROOT", DEFAULT_SITE_MEDIA_ROOT));
        let ttl_hours = read("CLOUD_SESSION_TTL_HOURS", "168")
            .parse::<u64>()
            .context("CLOUD_SESSION_TTL_HOURS 必须是整数")?;
        if ttl_hours == 0 || ttl_hours > 24 * 90 {
            bail!("CLOUD_SESSION_TTL_HOURS 必须在 1 到 2160 之间");
        }
        let environment = parse_environment(&read("CLOUD_ENVIRONMENT", "development"))?;
        Ok(Self {
            bind_addr,
            database_url,
            public_base_url,
            google_site_verification,
            baidu_site_verification,
            download_root,
            site_media_root,
            session_ttl: Duration::from_secs(ttl_hours * 3600),
            environment,
        })
    }
}

fn parse_environment(value: &str) -> Result<String> {
    match value {
        "development" | "staging" | "production" => Ok(value.to_owned()),
        _ => bail!("CLOUD_ENVIRONMENT 只能是 development、staging 或 production"),
    }
}

fn read(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("缺少必需环境变量 {name}"))
}

fn parse_public_base_url(value: &str) -> Result<Url> {
    let mut url = Url::parse(value).context("CLOUD_PUBLIC_BASE_URL 不是合法 URL")?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("CLOUD_PUBLIC_BASE_URL 只能使用 http 或 https 协议");
    }
    if url.host_str().is_none() {
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

fn read_site_verification(name: &str) -> Result<Option<String>> {
    let value = match env::var(name) {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => bail!("{name} 必须是有效 UTF-8 文本"),
    };
    parse_site_verification(name, value.as_deref())
}

fn parse_site_verification(name: &str, value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Ok(None);
    }
    if value.len() > 256
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("{name} 只能包含 ASCII 字母、数字、短横线或下划线，且长度必须在 1 到 256 之间");
    }
    Ok(Some(value.to_owned()))
}
