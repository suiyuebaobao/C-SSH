//! 校验设备登记输入并为当前账号创建设备。

use crate::{model::CreateDeviceOutcome, repository, validation, Platform};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use serde::Deserialize;

#[derive(Clone, Debug, Default)]
pub struct TrustedRequestMetadata {
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDevice {
    pub name: String,
    pub platform: Platform,
    pub public_id: String,
    #[serde(default)]
    pub client_version: Option<String>,
    #[serde(default)]
    pub device_fingerprint: Option<String>,
    #[serde(default)]
    pub physical_device_fingerprint: Option<String>,
}

impl CreateDevice {
    pub(crate) fn validate(self) -> AppResult<Self> {
        let validated = Self {
            name: validation::name(&self.name)?,
            platform: self.platform,
            public_id: validation::public_id(&self.public_id)?,
            client_version: validation::client_version(self.client_version.as_deref())?,
            device_fingerprint: validation::device_fingerprint(self.device_fingerprint.as_deref())?,
            physical_device_fingerprint: validation::device_fingerprint(
                self.physical_device_fingerprint.as_deref(),
            )?,
        };
        if let Some(physical) = validated.physical_device_fingerprint.as_ref() {
            if validated.device_fingerprint.as_ref() != Some(physical) {
                return Err(cloud_domain::AppError::Validation(
                    "设备指纹字段不一致".to_owned(),
                ));
            }
        }
        Ok(validated)
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    command: CreateDevice,
    metadata: TrustedRequestMetadata,
    quota: crate::DeviceQuotaResolver,
) -> AppResult<CreateDeviceOutcome> {
    let command = command.validate()?;
    repository::create::bind(
        pool,
        session,
        &command.name,
        command.platform.as_str(),
        &command.public_id,
        metadata.last_login_ip.as_deref(),
        metadata.user_agent.as_deref(),
        command.client_version.as_deref(),
        command.device_fingerprint.as_deref(),
        command.physical_device_fingerprint.as_deref(),
        quota,
    )
    .await
}
