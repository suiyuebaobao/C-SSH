//! 执行版本域 use-case 的第二层管理员身份校验。
//! HTTP handler 先从会话派生能力，本层再拒绝无效管理主体。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

pub(crate) fn require(actor: &AdminActor) -> AppResult<Uuid> {
    let account_id = actor.account_id();
    if account_id.is_nil() {
        return Err(AppError::Unauthorized("管理员身份无效".into()));
    }
    Ok(account_id)
}
