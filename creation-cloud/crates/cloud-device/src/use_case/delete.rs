//! 将当前账号尚未撤销的设备标记为撤销。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::repository;

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
) -> AppResult<()> {
    if repository::delete::revoke(pool, session.account_id, device_id).await? != 1 {
        return Err(AppError::NotFound("设备不存在或已撤销".to_owned()));
    }
    Ok(())
}
