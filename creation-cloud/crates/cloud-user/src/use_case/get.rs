//! 校验资料所有权后读取单个用户资料。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Profile, repository};

use super::ownership;

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account_id: Uuid,
) -> AppResult<Profile> {
    ownership::ensure(session.account_id, account_id)?;
    repository::get::find(pool, account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("用户资料不存在".to_owned()))
}
