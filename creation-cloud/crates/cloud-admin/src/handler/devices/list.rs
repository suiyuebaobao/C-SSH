//! 从服务端会话派生管理员身份并返回设备元数据分页。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page};

use crate::{AdminDevice, AdminDeviceListQuery, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminDeviceListQuery>,
) -> AppResult<Json<Page<AdminDevice>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_devices(&actor, query).await?))
}
