//! 从环境变量读取 Creation Cloud 启动配置并校验安全边界。

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::{Context, Result, bail};
use url::Url;

mod maintenance;
mod public_base_url;

#[cfg(test)]
mod public_base_url_tests;
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
    pub maintenance: MaintenanceConfig,
}

pub use maintenance::{MaintenanceConfig, TaskSchedule};

impl CloudConfig {
    pub fn from_env() -> Result<Self> {
        let configured_environment = read_optional("CLOUD_ENVIRONMENT")?;
        let environment =
            parse_environment(configured_environment.as_deref().unwrap_or("development"))?;
        let bind_addr = read("CLOUD_BIND_ADDR", "127.0.0.1:8088")
            .parse()
            .context("CLOUD_BIND_ADDR 不是合法监听地址")?;
        let database_url = required("CLOUD_DATABASE_URL")?;
        let configured_public_base_url = read_optional("CLOUD_PUBLIC_BASE_URL")?;
        let public_base_url =
            public_base_url::resolve(&environment, configured_public_base_url.as_deref())?;
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
        let maintenance = MaintenanceConfig::from_env()?;
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
            maintenance,
        })
    }
}

fn parse_environment(value: &str) -> Result<String> {
    match value {
        "development" | "test" | "staging" | "production" => Ok(value.to_owned()),
        _ => bail!("CLOUD_ENVIRONMENT 只能是 development、test、staging 或 production"),
    }
}

fn read(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("缺少必需环境变量 {name}"))
}

fn read_optional(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{name} 必须是有效 UTF-8 文本"),
    }
}

fn read_site_verification(name: &str) -> Result<Option<String>> {
    let value = read_optional(name)?;
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
