//! 从服务端会话派生管理员身份并返回脱敏用户分页。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page};

use crate::{AdminUser, AdminUserListQuery, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminUserListQuery>,
) -> AppResult<Json<Page<AdminUser>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_users(&actor, query).await?))
}
