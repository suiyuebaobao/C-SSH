//! 校验资料所有权后删除当前账号的资料记录。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::repository;

use super::ownership;

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account_id: Uuid,
) -> AppResult<()> {
    ownership::ensure(session.account_id, account_id)?;
    if repository::delete::remove(pool, account_id).await? != 1 {
        return Err(AppError::NotFound("用户资料不存在".to_owned()));
    }
    Ok(())
}
