//! 认证会话、期限元数据和对外安全视图。

use chrono::{DateTime, Utc};
use cloud_domain::AppResult;
pub use cloud_domain::AuthenticatedSession;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionMetadata {
    pub email_verified: bool,
    pub session_kind: String,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
}

impl SessionMetadata {
    pub(crate) fn unbound(expires_at: DateTime<Utc>, email_verified: bool) -> Self {
        Self {
            email_verified,
            session_kind: "unbound".to_owned(),
            idle_expires_at: expires_at,
            absolute_expires_at: expires_at,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct SessionView {
    pub account_id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub device_id: Option<Uuid>,
    pub session_kind: String,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub csrf_token: String,
}

impl SessionView {
    pub(crate) fn from_parts(session: &AuthenticatedSession, metadata: &SessionMetadata) -> Self {
        Self {
            account_id: session.account_id,
            email: (!session.email.is_empty()).then(|| session.email.clone()),
            email_verified: metadata.email_verified,
            admin_login_name: session.admin_login_name.clone(),
            role: session.role.clone(),
            device_id: session.device_id,
            session_kind: metadata.session_kind.clone(),
            idle_expires_at: metadata.idle_expires_at,
            absolute_expires_at: metadata.absolute_expires_at,
            csrf_token: session.csrf_token.clone(),
        }
    }
}

impl From<&AuthenticatedSession> for SessionView {
    fn from(session: &AuthenticatedSession) -> Self {
        Self::from_parts(
            session,
            &SessionMetadata {
                email_verified: !session.email.is_empty(),
                session_kind: if session.device_id.is_some() {
                    "device".to_owned()
                } else {
                    "unbound".to_owned()
                },
                idle_expires_at: session.expires_at,
                absolute_expires_at: session.expires_at,
            },
        )
    }
}

pub struct IssuedSession {
    pub raw_token: String,
    pub session: AuthenticatedSession,
    pub metadata: SessionMetadata,
}

impl IssuedSession {
    #[must_use]
    pub fn view(&self) -> SessionView {
        SessionView::from_parts(&self.session, &self.metadata)
    }

    /// Builds the secure browser cookie while keeping cookie policy in auth.
    pub fn set_cookie_header(&self) -> AppResult<axum::http::HeaderValue> {
        crate::cookie::session_header(&self.raw_token, self.metadata.idle_expires_at)
    }
}
