//! 已登录实例的稳定身份补登记；只允许当前设备，旧数据与会话令牌保持。
use crate::{repository, validation, Device};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindDeviceIdentity {
    pub physical_device_fingerprint: String,
}

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device: Uuid,
    input: BindDeviceIdentity,
) -> AppResult<Device> {
    if session.device_id != Some(device) || device.is_nil() {
        return Err(AppError::Forbidden("只能登记当前设备的身份".to_owned()));
    }
    let fingerprint = validation::device_fingerprint(Some(&input.physical_device_fingerprint))?
        .ok_or_else(|| AppError::Validation("设备指纹不能为空".to_owned()))?;
    repository::identity::bind(pool, session, device, &fingerprint).await
}
