//! 强制用户资料操作只能命中当前会话所属账号。

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

pub(crate) fn ensure(account_id: Uuid, target_id: Uuid) -> AppResult<()> {
    if account_id != target_id {
        return Err(AppError::Forbidden("不能操作其他账号的资料".to_owned()));
    }
    Ok(())
}
