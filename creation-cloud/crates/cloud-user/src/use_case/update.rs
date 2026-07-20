//! 校验资料所有权与变更字段后更新用户资料。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::{Profile, repository, validation};

use super::ownership;

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub display_name: Option<String>,
    pub locale: Option<String>,
}

impl UpdateProfile {
    pub(crate) fn validate(self) -> AppResult<Self> {
        if self.display_name.is_none() && self.locale.is_none() {
            return Err(AppError::Validation("至少提供一个变更字段".to_owned()));
        }
        Ok(Self {
            display_name: self
                .display_name
                .as_deref()
                .map(validation::display_name)
                .transpose()?,
            locale: self.locale.as_deref().map(validation::locale).transpose()?,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account_id: Uuid,
    command: UpdateProfile,
) -> AppResult<Profile> {
    ownership::ensure(session.account_id, account_id)?;
    let command = command.validate()?;
    repository::update::apply(
        pool,
        account_id,
        command.display_name.as_deref(),
        command.locale.as_deref(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("用户资料不存在".to_owned()))
}
