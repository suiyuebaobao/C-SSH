//! 管理员设备限额用例；只接受服务端会话身份，不影响现有会话与云端密文。
use crate::{repository, AccountDevicePolicy, DeviceQuotaResolver, UpdateDevicePolicy};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use cloud_store::PgPool;
use uuid::Uuid;

fn authorize(session: &AuthenticatedSession, account: Uuid) -> AppResult<Uuid> {
    let actor = AdminActor::from_session(session)?.account_id();
    if actor.is_nil() || account.is_nil() {
        return Err(AppError::Validation("账号身份无效".to_owned()));
    }
    Ok(actor)
}

pub(crate) async fn read(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account: Uuid,
    quota: DeviceQuotaResolver,
) -> AppResult<AccountDevicePolicy> {
    authorize(session, account)?;
    repository::policy::read(pool, account, quota).await
}

pub(crate) async fn update(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account: Uuid,
    input: UpdateDevicePolicy,
    quota: DeviceQuotaResolver,
) -> AppResult<AccountDevicePolicy> {
    let actor = authorize(session, account)?;
    let maximum = input.validated_maximum()?;
    repository::policy::update(
        pool,
        actor,
        account,
        maximum,
        input.expected_revision,
        quota,
    )
    .await
}
