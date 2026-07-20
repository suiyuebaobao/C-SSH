//! 创建当前账号缺失的用户资料。

use cloud_domain::AppResult;
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use serde::Deserialize;

use crate::{Profile, repository, validation};

#[derive(Debug, Deserialize)]
pub struct CreateProfile {
    pub display_name: String,
    pub locale: String,
}

impl CreateProfile {
    pub(crate) fn validate(self) -> AppResult<Self> {
        Ok(Self {
            display_name: validation::display_name(&self.display_name)?,
            locale: validation::locale(&self.locale)?,
        })
    }
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    command: CreateProfile,
) -> AppResult<Profile> {
    let command = command.validate()?;
    repository::create::insert(
        pool,
        session.account_id,
        &command.display_name,
        &command.locale,
    )
    .await
}
