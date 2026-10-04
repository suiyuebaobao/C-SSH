//! 为管理员网页详情提供原HTML已允许的完整邮箱，不改变既有脱敏用户API。
use crate::{AdminUser, Service};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(crate) struct Profile {
    #[serde(flatten)]
    user: AdminUser,
    email: String,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
) -> AppResult<Json<Profile>> {
    let actor = AdminActor::from_session(&session)?;
    let user = service.get_user(&actor, account_id).await?;
    let email = user.email_for_admin_page().to_owned();
    Ok(Json(Profile { user, email }))
}
