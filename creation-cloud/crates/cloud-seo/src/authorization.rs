use cloud_domain::{AdminActor, AppError, AppResult};
use uuid::Uuid;

pub(crate) fn require(actor: &AdminActor) -> AppResult<Uuid> {
    let account_id = actor.account_id();
    if account_id.is_nil() {
        return Err(AppError::Unauthorized("管理员身份无效".to_owned()));
    }
    Ok(account_id)
}
