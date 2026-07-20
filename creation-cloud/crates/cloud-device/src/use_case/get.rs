//! 在当前账号范围内读取单个设备。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{Device, repository};

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
) -> AppResult<Device> {
    repository::get::find(pool, session.account_id, device_id)
        .await?
        .ok_or_else(|| AppError::NotFound("设备不存在".to_owned()))
}
