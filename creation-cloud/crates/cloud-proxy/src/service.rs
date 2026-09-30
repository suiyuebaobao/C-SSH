//! 编排节点目录、目标绑定短凭据和内网授权，不让秘密进入日志或持久化明文。

use std::collections::BTreeMap;

use chrono::Duration;
use cloud_config::ProxyConfig;
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    ProxyError, TargetProtocol, crypto,
    model::{
        AuthorizeRequest, AuthorizeResponse, CloseTunnelRequest, CredentialIssueRequest,
        CredentialIssueResponse, NodeEndpointView, NodeListResponse, NodeView,
    },
    repository::{self, NewCredential},
    validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    environment: String,
    config: ProxyConfig,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool, environment: String, config: ProxyConfig) -> Self {
        Self {
            pool,
            environment,
            config,
        }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.is_enabled()
    }

    pub async fn list_nodes(
        &self,
        session: &AuthenticatedSession,
        target_protocol: TargetProtocol,
    ) -> Result<NodeListResponse, ProxyError> {
        self.ensure_enabled()?;
        let device_id = session.device_id.ok_or(ProxyError::DeviceRequired)?;
        let records =
            repository::list_nodes(&self.pool, session, device_id, &self.environment).await?;
        self.node_directory(records, target_protocol)
    }

    fn node_directory(
        &self,
        records: Vec<repository::NodeEndpointRecord>,
        target_protocol: TargetProtocol,
    ) -> Result<NodeListResponse, ProxyError> {
        let mut nodes = BTreeMap::<Uuid, NodeView>::new();
        for record in records {
            if !self.supports_protocol(record.node_id, target_protocol) {
                continue;
            }
            let port = u16::try_from(record.port).map_err(|_| ProxyError::Internal)?;
            let node = nodes.entry(record.node_id).or_insert_with(|| NodeView {
                id: record.node_id,
                display_name: record.display_name,
                region: record.region,
                endpoints: Vec::new(),
            });
            node.endpoints.push(NodeEndpointView {
                protocol: record.protocol,
                host: record.host,
                port,
                tls_server_name: record.tls_server_name,
                target_protocols: (target_protocol == TargetProtocol::Rdp)
                    .then_some([TargetProtocol::Ssh, TargetProtocol::Rdp]),
            });
        }
        Ok(NodeListResponse {
            nodes: nodes.into_values().collect(),
        })
    }

    pub async fn issue_credential(
        &self,
        session: &AuthenticatedSession,
        request: CredentialIssueRequest,
    ) -> Result<CredentialIssueResponse, ProxyError> {
        self.ensure_enabled()?;
        if request.mutation_id.is_nil() || request.node_id.is_nil() {
            return Err(ProxyError::InvalidRequest);
        }
        let device_id = session.device_id.ok_or(ProxyError::DeviceRequired)?;
        let target = validation::target(&request.target)?;
        if !self.supports_protocol(request.node_id, target.protocol) {
            return Err(ProxyError::Unavailable);
        }
        let (key_version, key) = self.config.active_key().ok_or(ProxyError::Unavailable)?;
        let target_digest = crypto::target_digest(&key, &target);
        let request_digest = crypto::request_digest(&key, request.node_id, &target_digest);
        let proposed_id = Uuid::now_v7();
        let username = crypto::credential_username(proposed_id);
        let password = crypto::credential_password(&key, proposed_id);
        let password_digest = crypto::password_digest(&key, &password);
        let issued = repository::issue(
            &self.pool,
            session,
            device_id,
            &self.environment,
            NewCredential {
                id: proposed_id,
                mutation_id: request.mutation_id,
                node_id: request.node_id,
                username: &username,
                key_version,
                password_digest: &password_digest,
                target_digest: &target_digest,
                request_digest: &request_digest,
                ttl_seconds: i64::try_from(self.config.credential_ttl.as_secs())
                    .map_err(|_| ProxyError::Internal)?,
            },
        )
        .await?;
        let issued_key = self
            .config
            .key(issued.key_version)
            .ok_or(ProxyError::Unavailable)?;
        let password = crypto::credential_password(&issued_key, issued.id);
        let lifetime = issued.expires_at - issued.issued_at;
        let refresh_after =
            issued.issued_at + Duration::seconds(lifetime.num_seconds().saturating_mul(4) / 5);
        Ok(CredentialIssueResponse {
            credential_id: issued.id,
            node_id: issued.node_id,
            username: issued.username,
            password,
            generation: issued.generation,
            issued_at: issued.issued_at,
            refresh_after,
            expires_at: issued.expires_at,
        })
    }

    pub async fn revoke_credential(
        &self,
        session: &AuthenticatedSession,
        credential_id: Uuid,
    ) -> Result<(), ProxyError> {
        self.ensure_enabled()?;
        if credential_id.is_nil() {
            return Err(ProxyError::InvalidRequest);
        }
        let device_id = session.device_id.ok_or(ProxyError::DeviceRequired)?;
        repository::revoke(&self.pool, session, device_id, credential_id).await
    }

    pub(crate) async fn authorize(
        &self,
        node_token: &str,
        request: AuthorizeRequest,
    ) -> Result<AuthorizeResponse, ProxyError> {
        self.ensure_enabled()?;
        let token_digest = crypto::node_token_digest(node_token);
        let lease_expires_at = match request {
            AuthorizeRequest::Admission {
                connection_id,
                username,
                password,
                target_host,
                target_port,
                target_protocol,
            } => {
                if connection_id.is_nil() {
                    return Err(ProxyError::AuthorizationDenied);
                }
                validation::internal_credential(&username, &password)?;
                let target =
                    validation::authorize_target(&target_host, target_port, target_protocol)?;
                let candidate = repository::admission_candidate(
                    &self.pool,
                    &token_digest,
                    &username,
                    &self.environment,
                )
                .await?
                .ok_or(ProxyError::AuthorizationDenied)?;
                if !self.supports_protocol(candidate.node_id, target_protocol) {
                    return Err(ProxyError::AuthorizationDenied);
                }
                let key = self
                    .config
                    .key(candidate.key_version)
                    .ok_or(ProxyError::AuthorizationDenied)?;
                let password_digest = crypto::password_digest(&key, &password);
                let target_digest = crypto::target_digest(&key, &target);
                if !crypto::matches(&candidate.password_digest, &password_digest)
                    || !crypto::matches(&candidate.target_digest, &target_digest)
                {
                    return Err(ProxyError::AuthorizationDenied);
                }
                repository::admit_tunnel(
                    &self.pool,
                    &token_digest,
                    &self.environment,
                    connection_id,
                    &candidate,
                    &password_digest,
                    &target_digest,
                )
                .await?
                .ok_or(ProxyError::AuthorizationDenied)?
            }
            AuthorizeRequest::Continuation { connection_id } => {
                if connection_id.is_nil() {
                    return Err(ProxyError::AuthorizationDenied);
                }
                repository::continuation_lease(
                    &self.pool,
                    &token_digest,
                    &self.environment,
                    connection_id,
                )
                .await?
                .ok_or(ProxyError::AuthorizationDenied)?
            }
        };
        Ok(AuthorizeResponse {
            authorized: true,
            lease_expires_at,
        })
    }

    pub(crate) async fn close_tunnel(
        &self,
        node_token: &str,
        request: CloseTunnelRequest,
    ) -> Result<(), ProxyError> {
        self.ensure_enabled()?;
        if request.connection_id.is_nil() {
            return Err(ProxyError::AuthorizationDenied);
        }
        let token_digest = crypto::node_token_digest(node_token);
        repository::close_tunnel(
            &self.pool,
            &token_digest,
            &self.environment,
            request.connection_id,
        )
        .await
    }

    fn ensure_enabled(&self) -> Result<(), ProxyError> {
        if !self.config.is_enabled() {
            return Err(ProxyError::Unavailable);
        }
        Ok(())
    }

    fn supports_protocol(&self, node_id: Uuid, protocol: TargetProtocol) -> bool {
        protocol == TargetProtocol::Ssh || self.config.rdp_node_ids.contains(&node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn directory_preserves_ssh_shape_and_advertises_only_opted_in_rdp_nodes() {
        let upgraded = Uuid::from_u128(1);
        let legacy = Uuid::from_u128(2);
        let mut config = ProxyConfig::default();
        config.rdp_node_ids.insert(upgraded);
        let pool = PgPool::connect_lazy("postgres://example:example@example.com/example").unwrap();
        let service = Service::new(pool, "test".to_owned(), config);
        let records = || {
            [upgraded, legacy]
                .into_iter()
                .map(|node_id| repository::NodeEndpointRecord {
                    node_id,
                    display_name: "Test".to_owned(),
                    region: "test".to_owned(),
                    protocol: "http_connect".to_owned(),
                    host: "proxy.example.com".to_owned(),
                    port: 443,
                    tls_server_name: "proxy.example.com".to_owned(),
                })
                .collect()
        };
        let ssh = service
            .node_directory(records(), TargetProtocol::Ssh)
            .unwrap();
        assert_eq!(ssh.nodes.len(), 2);
        let json = serde_json::to_value(&ssh).unwrap();
        for node in json["nodes"].as_array().unwrap() {
            let endpoint = node["endpoints"][0].as_object().unwrap();
            assert_eq!(endpoint.len(), 4);
            assert!(!endpoint.contains_key("target_protocols"));
        }
        let rdp = service
            .node_directory(records(), TargetProtocol::Rdp)
            .unwrap();
        assert_eq!(rdp.nodes.len(), 1);
        assert_eq!(rdp.nodes[0].id, upgraded);
        assert_eq!(
            serde_json::to_value(&rdp).unwrap()["nodes"][0]["endpoints"][0]["target_protocols"],
            serde_json::json!(["ssh", "rdp"])
        );
        assert!(!service.supports_protocol(legacy, TargetProtocol::Rdp));
    }
}
