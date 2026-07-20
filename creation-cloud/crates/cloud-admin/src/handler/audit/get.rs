//! 返回指定不可变审计事件。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{AuditEvent, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
) -> AppResult<Json<AuditEvent>> {
    let actor = AdminActor::from_session(&session)?;
    Ok(Json(service.get_audit_event(&actor, event_id).await?))
}
