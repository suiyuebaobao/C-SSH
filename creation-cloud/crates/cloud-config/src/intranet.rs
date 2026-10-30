//! 显式内网部署的开关及独立验证码密钥；默认不改变官方云配置。

use anyhow::{Context, Result, bail};
use std::{fmt, fs, path::Path};

#[derive(Clone)]
pub struct IntranetConfig {
    captcha_key: [u8; 64],
    http: bool,
}

impl fmt::Debug for IntranetConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntranetConfig")
            .field("captcha_key", &"[redacted]")
            .field("http", &self.http)
            .finish()
    }
}

impl IntranetConfig {
    pub(crate) fn from_env() -> Result<Option<Self>> {
        let enabled = parse_mode(super::read_optional("CLOUD_INTRANET_MODE")?.as_deref())?;
        let http = parse_transport(super::read_optional("CLOUD_INTRANET_TRANSPORT")?.as_deref())?;
        if !enabled {
            if http {
                bail!("HTTP传输必须显式启用内网模式");
            }
            return Ok(None);
        }
        let path = super::required("CLOUD_EMAIL_CODE_HMAC_KEY_FILE")?;
        let metadata = fs::symlink_metadata(Path::new(&path)).context("内网验证码密钥不可读")?;
        if !metadata.file_type().is_file() || metadata.len() != 64 {
            bail!("内网验证码密钥必须是64字节普通文件");
        }
        let key = fs::read(path).context("内网验证码密钥不可读")?;
        if !key.iter().all(u8::is_ascii_hexdigit) {
            bail!("内网验证码密钥格式无效");
        }
        Ok(Some(Self {
            http,
            captcha_key: key
                .try_into()
                .map_err(|_| anyhow::anyhow!("内网验证码密钥长度无效"))?,
        }))
    }

    pub fn captcha_key(&self) -> [u8; 64] {
        self.captcha_key
    }

    pub fn uses_http(&self) -> bool {
        self.http
    }
}

fn parse_transport(value: Option<&str>) -> Result<bool> {
    match value {
        None | Some("https") => Ok(false),
        Some("http") => Ok(true),
        _ => bail!("CLOUD_INTRANET_TRANSPORT 只能为http或https"),
    }
}

fn parse_mode(value: Option<&str>) -> Result<bool> {
    match value {
        None | Some("0") => Ok(false),
        Some("1") => Ok(true),
        _ => bail!("CLOUD_INTRANET_MODE 只能为0或1"),
    }
}

