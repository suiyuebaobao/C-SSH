//! 在反馈用例层再次收敛用户与管理员身份，避免仅依赖路由可见性授权。

use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

pub(crate) fn user(session: &AuthenticatedSession) -> AppResult<Uuid> {
    if session.account_id.is_nil() {
        return Err(AppError::Unauthorized("反馈账号身份无效".to_owned()));
    }
    Ok(session.account_id)
}

pub(crate) fn admin(actor: &AdminActor) -> AppResult<Uuid> {
    if actor.account_id().is_nil() {
        return Err(AppError::Unauthorized("管理员身份无效".to_owned()));
    }
    Ok(actor.account_id())
}
