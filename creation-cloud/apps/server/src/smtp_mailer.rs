//! 基于 SMTP 的验证码投递适配器。错误对外只暴露稳定的可重试语义。

use std::{future::Future, pin::Pin, time::Duration};

use cloud_auth::VerificationMailer;
use cloud_config::{SmtpConfig, SmtpSecurity};
use cloud_domain::{AppError, AppResult};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

#[derive(Clone)]
pub struct SmtpVerificationMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpVerificationMailer {
    pub fn new(config: &SmtpConfig) -> anyhow::Result<Self> {
        let builder = match config.security {
            SmtpSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?,
            SmtpSecurity::StartTls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
            }
        };
        let transport = builder
            .port(config.port)
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
    ) -> Pin<Box<dyn Future<Output = AppResult<()>> + Send + 'a>> {
        Box::pin(async move { self.send(email, code).await })
    }
}

impl SmtpVerificationMailer {
    async fn send(&self, recipient: &str, code: &str) -> AppResult<()> {
        let recipient = recipient
            .parse::<Mailbox>()
            .map_err(|_| AppError::Validation("收件邮箱地址无效".to_owned()))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject("Creation-SSH 邮箱验证码")
            .header(ContentType::TEXT_PLAIN)
            .body(format!(
                "你的 Creation-SSH 验证码是：{code}\n\n验证码 10 分钟内有效，请勿转发给其他人。"
            ))
            .map_err(|_| AppError::Internal("验证码邮件构建失败".to_owned()))?;
        self.transport
            .send(message)
            .await
            .map_err(|_| AppError::Unavailable("验证码邮件发送失败，请稍后重试".to_owned()))?;
        Ok(())
    }
}
