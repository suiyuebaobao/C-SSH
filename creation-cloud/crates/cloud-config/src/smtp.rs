//! 邮箱验证码投递配置。凭据与验证码摘要密钥只允许从文件读取。

use std::{env, fmt, fs, path::PathBuf, sync::Arc};

use anyhow::{Context, Result, bail};

const DEFAULT_SMTP_PORT: u16 = 465;
const MIN_VERIFICATION_KEY_BYTES: usize = 32;
const MAX_SECRET_FILE_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmtpSecurity {
    Tls,
    StartTls,
}

#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub security: SmtpSecurity,
    pub username: String,
    pub from_address: String,
    password: Arc<str>,
    verification_key: Arc<[u8]>,
}

impl fmt::Debug for SmtpConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SmtpConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("security", &self.security)
            .field("username", &"[redacted]")
            .field("from_address", &self.from_address)
            .field("password", &"[redacted]")
            .field("verification_key", &"[redacted]")
            .finish()
    }
}

impl SmtpConfig {
    pub(crate) fn from_env() -> Result<Option<Self>> {
        let host = optional_env("CLOUD_SMTP_HOST")?;
        let username = optional_env("CLOUD_SMTP_USERNAME")?;
        let from_address = optional_env("CLOUD_SMTP_FROM")?;
        let password_file = optional_env("CLOUD_SMTP_PASSWORD_FILE")?;
        let verification_key_file = optional_env("CLOUD_EMAIL_CODE_HMAC_KEY_FILE")?;
        let port = optional_env("CLOUD_SMTP_PORT")?;
        let security = optional_env("CLOUD_SMTP_SECURITY")?;

        let has_any = [
            host.as_ref(),
            username.as_ref(),
            from_address.as_ref(),
            password_file.as_ref(),
            verification_key_file.as_ref(),
            port.as_ref(),
            security.as_ref(),
        ]
        .into_iter()
        .any(|value| value.is_some());
        if !has_any {
            return Ok(None);
        }

        let host = required_value("CLOUD_SMTP_HOST", host)?;
        let username = required_value("CLOUD_SMTP_USERNAME", username)?;
        let from_address = required_value("CLOUD_SMTP_FROM", from_address)?;
        let password_path =
            PathBuf::from(required_value("CLOUD_SMTP_PASSWORD_FILE", password_file)?);
        let verification_key_path = PathBuf::from(required_value(
            "CLOUD_EMAIL_CODE_HMAC_KEY_FILE",
            verification_key_file,
        )?);
        let port = port
            .unwrap_or_else(|| DEFAULT_SMTP_PORT.to_string())
            .parse::<u16>()
            .context("CLOUD_SMTP_PORT 必须是 1 到 65535 的整数")?;
        if port == 0 {
            bail!("CLOUD_SMTP_PORT 必须是 1 到 65535 的整数");
        }
        let security = parse_security(security.as_deref().unwrap_or("tls"))?;
        validate_identity("CLOUD_SMTP_USERNAME", &username)?;
        validate_mailbox("CLOUD_SMTP_FROM", &from_address)?;

        let password = read_secret_file("CLOUD_SMTP_PASSWORD_FILE", &password_path)?;
        let verification_key =
            read_secret_file("CLOUD_EMAIL_CODE_HMAC_KEY_FILE", &verification_key_path)?;
        if verification_key.len() < MIN_VERIFICATION_KEY_BYTES {
            bail!("CLOUD_EMAIL_CODE_HMAC_KEY_FILE 内容至少需要 {MIN_VERIFICATION_KEY_BYTES} 字节");
        }

        Ok(Some(Self {
            host,
            port,
            security,
            username,
            from_address,
            password: Arc::from(password),
            verification_key: Arc::from(verification_key.into_bytes()),
        }))
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub fn verification_key(&self) -> &[u8] {
        &self.verification_key
    }
}

fn optional_env(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{name} 必须是有效 UTF-8 文本"),
    }
}

fn required_value(name: &str, value: Option<String>) -> Result<String> {
    value.with_context(|| format!("SMTP 已启用但缺少 {name}"))
}

fn parse_security(value: &str) -> Result<SmtpSecurity> {
    match value {
        "tls" => Ok(SmtpSecurity::Tls),
        "starttls" => Ok(SmtpSecurity::StartTls),
        _ => bail!("CLOUD_SMTP_SECURITY 只能是 tls 或 starttls"),
    }
}

fn validate_mailbox(name: &str, value: &str) -> Result<()> {
    validate_identity(name, value)?;
    if !value.contains('@') {
        bail!("{name} 不是合法邮箱地址");
    }
    Ok(())
}

fn validate_identity(name: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 320
        || value.trim() != value
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        bail!("{name} 不是合法 SMTP 账号标识");
    }
    Ok(())
}

fn read_secret_file(name: &str, path: &PathBuf) -> Result<String> {
    let metadata = fs::metadata(path).with_context(|| format!("{name} 指向的文件不可读取"))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_SECRET_FILE_BYTES {
        bail!("{name} 指向的文件必须是非空且不超过 64 KiB 的普通文件");
    }
    let value = fs::read_to_string(path)
        .with_context(|| format!("{name} 指向的文件必须是有效 UTF-8 文本"))?;
    let value = value.trim_end_matches(['\r', '\n']).to_owned();
    if value.is_empty() {
        bail!("{name} 指向的文件不能为空");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smtp_security_has_a_closed_vocabulary() {
        assert_eq!(parse_security("tls").expect("tls"), SmtpSecurity::Tls);
        assert_eq!(
            parse_security("starttls").expect("starttls"),
            SmtpSecurity::StartTls
        );
        for invalid in ["", "plain", "TLS", "opportunistic"] {
            assert!(parse_security(invalid).is_err());
        }
    }

    #[test]
    fn smtp_username_accepts_provider_account_identifiers() {
        validate_identity("CLOUD_SMTP_USERNAME", "123456789").expect("numeric account");
        validate_identity("CLOUD_SMTP_USERNAME", "mailer@example.com").expect("mailbox account");
        for invalid in ["", " leading", "trailing ", "line\nbreak"] {
            assert!(validate_identity("CLOUD_SMTP_USERNAME", invalid).is_err());
        }
    }

    #[test]
    fn smtp_debug_output_never_contains_secrets() {
        let config = SmtpConfig {
            host: "smtp.example.com".to_owned(),
            port: 465,
            security: SmtpSecurity::Tls,
            username: "mailer@example.com".to_owned(),
            from_address: "noreply@example.com".to_owned(),
            password: Arc::from("password-marker"),
            verification_key: Arc::from(b"verification-key-marker".as_slice()),
        };
        let output = format!("{config:?}");
        assert!(!output.contains("password-marker"));
        assert!(!output.contains("verification-key-marker"));
        assert!(!output.contains("mailer@example.com"));
    }
}
