//! 返回审计结果只读概览。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{SecurityAuditOverview, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<SecurityAuditOverview>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.security_audit_overview(&actor).await?))
}
