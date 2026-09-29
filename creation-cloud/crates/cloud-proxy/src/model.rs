//! 定义公网目录、短凭据请求和内网鉴权的最小 DTO。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetProtocol {
    #[default]
    Ssh,
    Rdp,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeListQuery {
    #[serde(default)]
    pub target_protocol: TargetProtocol,
}

#[derive(Deserialize)]
pub struct CredentialIssueRequest {
    pub mutation_id: Uuid,
    pub node_id: Uuid,
    pub target: ProxyTarget,
}

#[derive(Deserialize)]
pub struct ProxyTarget {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: TargetProtocol,
}

#[derive(Serialize)]
pub struct CredentialIssueResponse {
    pub credential_id: Uuid,
    pub node_id: Uuid,
    pub username: String,
    pub password: String,
    pub generation: i64,
    pub issued_at: DateTime<Utc>,
    pub refresh_after: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct NodeListResponse {
    pub nodes: Vec<NodeView>,
}

#[derive(Serialize)]
pub struct NodeView {
    pub id: Uuid,
    pub display_name: String,
    pub region: String,
    pub endpoints: Vec<NodeEndpointView>,
}

#[derive(Serialize)]
pub struct NodeEndpointView {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub tls_server_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_protocols: Option<[TargetProtocol; 2]>,
}

#[derive(Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuthorizeRequest {
    Admission {
        connection_id: Uuid,
        username: String,
        password: String,
        target_host: String,
        target_port: u16,
        #[serde(default)]
        target_protocol: TargetProtocol,
    },
    Continuation {
        connection_id: Uuid,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloseTunnelRequest {
    pub connection_id: Uuid,
}

#[derive(Serialize)]
pub(crate) struct AuthorizeResponse {
    pub authorized: bool,
    pub lease_expires_at: DateTime<Utc>,
}
