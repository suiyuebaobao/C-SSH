//! 执行下载管理 use-case 的第二层管理员身份校验。
//! 公开下载用例不依赖该能力类型，保持公开路由边界不变。

use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

pub(crate) fn require(actor: &AdminActor) -> AppResult<Uuid> {
    let account_id = actor.account_id();
    if account_id.is_nil() {
        return Err(AppError::Unauthorized("管理员身份无效".into()));
    }
    Ok(account_id)
}
