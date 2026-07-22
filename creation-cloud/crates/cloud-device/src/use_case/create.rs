//! 校验设备登记输入并为当前账号创建设备。

use crate::{Platform, model::CreateDeviceOutcome, repository, validation};
use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateDevice {
    pub name: String,
    pub platform: Platform,
    pub public_id: String,
}

impl CreateDevice {
    pub(crate) fn validate(self) -> AppResult<Self> {
        Ok(Self {
            name: validation::name(&self.name)?,
            platform: self.platform,
            public_id: validation::public_id(&self.public_id)?,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    command: CreateDevice,
) -> AppResult<CreateDeviceOutcome> {
    let command = command.validate()?;
    repository::create::bind(
        pool,
        session,
        &command.name,
        command.platform.as_str(),
        &command.public_id,
    )
    .await
}
