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
    pub device_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub device_fingerprint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl SessionMetadata {
    pub(crate) fn unbound_with_request(
        expires_at: DateTime<Utc>,
        email_verified: bool,
        request: &crate::TrustedRequestMetadata,
    ) -> Self {
        let now = Utc::now();
        Self {
            email_verified,
            session_kind: "unbound".to_owned(),
            device_name: None,
            last_login_ip: request.last_login_ip.clone(),
            user_agent: request.user_agent.clone(),
            client_version: None,
            device_fingerprint: None,
            created_at: now,
            last_seen_at: now,
            idle_expires_at: expires_at,
            absolute_expires_at: expires_at,
            revoked_at: None,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct SessionView {
    pub session_id: Uuid,
    pub account_id: Uuid,
    pub email: Option<String>,
    pub email_verified: bool,
    pub admin_login_name: Option<String>,
    pub role: String,
    pub status: String,
    pub is_current: bool,
    pub device_id: Option<Uuid>,
    pub device_name: Option<String>,
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub device_fingerprint: Option<String>,
    pub session_kind: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub csrf_token: String,
}

impl SessionView {
    pub(crate) fn from_parts(session: &AuthenticatedSession, metadata: &SessionMetadata) -> Self {
        Self {
            session_id: session.session_id,
            account_id: session.account_id,
            email: (!session.email.is_empty()).then(|| session.email.clone()),
            email_verified: metadata.email_verified,
            admin_login_name: session.admin_login_name.clone(),
            role: session.role.clone(),
            status: "online".to_owned(),
            is_current: true,
            device_id: session.device_id,
            device_name: metadata.device_name.clone(),
            last_login_ip: metadata.last_login_ip.clone(),
            user_agent: metadata.user_agent.clone(),
            client_version: metadata.client_version.clone(),
            device_fingerprint: metadata.device_fingerprint.clone(),
            session_kind: metadata.session_kind.clone(),
            created_at: metadata.created_at,
            last_seen_at: metadata.last_seen_at,
            idle_expires_at: metadata.idle_expires_at,
            absolute_expires_at: metadata.absolute_expires_at,
            revoked_at: metadata.revoked_at,
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
                device_name: None,
                last_login_ip: None,
                user_agent: None,
                client_version: None,
                device_fingerprint: None,
                created_at: Utc::now(),
                last_seen_at: Utc::now(),
                idle_expires_at: session.expires_at,
                absolute_expires_at: session.expires_at,
                revoked_at: None,
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
