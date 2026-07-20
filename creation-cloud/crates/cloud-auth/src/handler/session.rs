//! 返回认证中间件已验证的当前会话视图。

use axum::{Extension, Json};

use crate::{AuthenticatedSession, SessionView};

pub(crate) async fn handle(
    Extension(session): Extension<AuthenticatedSession>,
) -> Json<SessionView> {
    Json(SessionView::from(&session))
}
