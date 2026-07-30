//! 返回认证中间件已验证的当前会话视图。

use axum::{Extension, Json};

use crate::{AuthenticatedSession, SessionMetadata, SessionView};

pub(crate) async fn handle(
    Extension(session): Extension<AuthenticatedSession>,
    Extension(metadata): Extension<SessionMetadata>,
) -> Json<SessionView> {
    Json(SessionView::from_parts(&session, &metadata))
}
