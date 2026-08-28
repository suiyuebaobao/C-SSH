//! 基于 SMTP 的验证码投递适配器。错误对外只暴露稳定的可重试语义。

use std::{future::Future, pin::Pin, time::Duration};

use cloud_auth::{VerificationMailer, VerificationPurpose};
use cloud_config::{SmtpConfig, SmtpSecurity};
use cloud_domain::{AppError, AppResult};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{
        Error as SmtpError,
        authentication::{Credentials, Mechanism},
    },
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SmtpVerificationMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpVerificationMailer {
    pub fn new(config: &SmtpConfig, _public_base_url: &str) -> anyhow::Result<Self> {
        let builder = match config.security {
            SmtpSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?,
            SmtpSecurity::StartTls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
            }
        };
        let transport = builder
            .port(config.port)
            .authentication(vec![Mechanism::Login, Mechanism::Plain])
            .credentials(Credentials::new(
                config.username.clone(),
                config.password().to_owned(),
            ))
            .timeout(Some(Duration::from_secs(15)))
            .build();
        let from = config.from_address.parse::<Mailbox>()?;
        Ok(Self { transport, from })
    }
}

impl VerificationMailer for SmtpVerificationMailer {
    fn send_verification<'a>(
        &'a self,
        email: &'a str,
        code: &'a str,
        purpose: VerificationPurpose,
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(async move { self.send(email, code, purpose).await })
    }

    fn send_password_reset<'a>(
        &'a self,
        email: &'a str,
        code: &'a str,
        _challenge_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.send(email, code, VerificationPurpose::PasswordReset)
                .await
        })
    }
}

impl cloud_host::ProtectionResetMailer for SmtpVerificationMailer {
    fn send_protection_reset<'a>(
        &'a self,
        email: &'a str,
        code: &'a str,
    ) -> cloud_host::ProtectionResetMailerFuture<'a> {
        Box::pin(async move {
            self.send_named(
                email,
                code,
                "Creation-SSH 数据保护清空验证码",
                "清空 Cloud 数据保护",
            )
            .await
        })
    }
}

impl SmtpVerificationMailer {
    async fn send(
        &self,
        recipient: &str,
        code: &str,
        purpose: VerificationPurpose,
    ) -> AppResult<()> {
        let subject = match purpose {
            VerificationPurpose::Registration => "Creation-SSH 邮箱验证码",
            VerificationPurpose::Login => "Creation-SSH 登录验证码",
            VerificationPurpose::PasswordReset => "Creation-SSH 找回密码验证码",
        };
        let action = match purpose {
            VerificationPurpose::Registration => "邮箱验证",
            VerificationPurpose::Login => "登录",
            VerificationPurpose::PasswordReset => "找回密码",
        };
        self.send_named(recipient, code, subject, action).await
    }

    async fn send_named(
        &self,
        recipient: &str,
        code: &str,
        subject: &str,
        action: &str,
    ) -> AppResult<()> {
        let recipient = recipient
            .parse::<Mailbox>()
            .map_err(|_| AppError::Validation("收件邮箱地址无效".to_owned()))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(verification_body(action, code))
            .map_err(|_| AppError::Internal("验证码邮件构建失败".to_owned()))?;
        self.transport.send(message).await.map_err(|error| {
            log_smtp_failure("verification", &error);
            AppError::Unavailable("验证码邮件发送失败，请稍后重试".to_owned())
        })?;
        Ok(())
    }
}

fn verification_body(action: &str, code: &str) -> String {
    format!(
        "你的 Creation-SSH {action}验证码是：{code}\n\n验证码 10 分钟内有效，请勿转发给其他人。"
    )
}

fn log_smtp_failure(purpose: &'static str, error: &SmtpError) {
    tracing::warn!(
        event = "smtp_delivery",
        stage = "send",
        result = "error",
        purpose,
        error_class = smtp_error_class(error),
    );
}

fn smtp_error_class(error: &SmtpError) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_tls() {
        "tls"
    } else if error.is_transient() {
        "transient"
    } else if error.is_permanent() {
        "permanent"
    } else if error.is_response() {
        "response"
    } else if error.is_client() {
        "client"
    } else if error.is_transport_shutdown() {
        "transport_shutdown"
    } else {
        "network_or_connection"
    }
}

#[cfg(test)]
mod tests {
    use super::verification_body;

    #[test]
    fn password_reset_mail_contains_only_the_code_and_safety_hint() {
        let body = verification_body("找回密码", "422216");
        assert!(body.contains("422216"));
        assert!(body.contains("10 分钟"));
        assert!(!body.contains("http"));
        assert!(!body.contains("/reset-password"));
        assert!(!body.contains("challenge_id"));
        assert!(!body.contains("专属链接"));
    }
}
