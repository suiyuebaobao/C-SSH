//! 校验设备新名称并只更新当前账号尚未撤销的设备。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{Device, repository, validation};

#[derive(Debug, Deserialize)]
pub struct UpdateDevice {
    pub name: String,
}

impl UpdateDevice {
    pub(crate) fn validate(self) -> AppResult<Self> {
        Ok(Self {
            name: validation::name(&self.name)?,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
    command: UpdateDevice,
) -> AppResult<Device> {
    let command = command.validate()?;
    repository::update::rename(pool, session.account_id, device_id, &command.name)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在或已撤销".to_owned()))
}
