//! 返回分页审计时间线。

use axum::{Extension, Json, extract::Query, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{AuditEvent, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Page<AuditEvent>>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.list_audit_events(&actor, query).await?))
}
