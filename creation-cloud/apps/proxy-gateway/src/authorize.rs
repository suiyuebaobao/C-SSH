//! 通过回环 HTTP 执行 admission、continuation 和 best-effort close；非明确 allow 一律失败。

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time,
};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    config::AuthorizationEndpoint,
    connect_request::{ConnectRequest, TargetProtocol},
};

const MAX_RESPONSE_HEADER_BYTES: usize = 8 * 1024;
const MAX_RESPONSE_BODY_BYTES: usize = 4 * 1024;
const MAX_AUTHORIZATION_LEASE_SECONDS: i64 = 15;

#[derive(Debug, Error)]
pub(crate) enum AuthorizationError {
    #[error("authorization denied")]
    Denied,
    #[error("authorization service unavailable")]
    Unavailable,
}

pub(crate) struct Authorization {
    pub lease_expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub(crate) struct Client {
    endpoint: AuthorizationEndpoint,
    node_token: Arc<Zeroizing<String>>,
    timeout: Duration,
}

impl Client {
    pub fn new(
        endpoint: AuthorizationEndpoint,
        node_token: Arc<Zeroizing<String>>,
        timeout: Duration,
    ) -> Self {
        Self {
            endpoint,
            node_token,
            timeout,
        }
    }

    pub async fn admit(
        &self,
        request: &ConnectRequest,
        connection_id: Uuid,
    ) -> Result<Authorization, AuthorizationError> {
        let body = AuthorizeRequest::Admission {
            connection_id,
            username: &request.credentials.username,
            password: &request.credentials.password,
            target_host: &request.host,
            target_port: request.port,
            target_protocol: request.protocol,
        };
        let response = self
            .post_json(self.endpoint.authorization_path, &body)
            .await?;
        parse_authorization_response(&response, Utc::now())
    }

    pub async fn continue_lease(
        &self,
        connection_id: Uuid,
    ) -> Result<Authorization, AuthorizationError> {
        let body = AuthorizeRequest::Continuation { connection_id };
        let response = self
            .post_json(self.endpoint.authorization_path, &body)
            .await?;
        parse_authorization_response(&response, Utc::now())
    }

    pub async fn close(&self, connection_id: Uuid) -> Result<(), AuthorizationError> {
        let response = self
            .post_json(self.endpoint.close_path, &CloseRequest { connection_id })
            .await?;
        parse_close_response(&response)
    }

    async fn post_json<T: Serialize>(
        &self,
        path: &str,
        request: &T,
    ) -> Result<Vec<u8>, AuthorizationError> {
        time::timeout(self.timeout, self.post_json_inner(path, request))
            .await
            .map_err(|_| AuthorizationError::Unavailable)?
    }

    async fn post_json_inner<T: Serialize>(
        &self,
        path: &str,
        request: &T,
    ) -> Result<Vec<u8>, AuthorizationError> {
        let body = Zeroizing::new(
            serde_json::to_vec(request).map_err(|_| AuthorizationError::Unavailable)?,
        );
        let prefix = Zeroizing::new(format!(
            "POST {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.endpoint.host_header,
            self.node_token.as_str(),
            body.len()
        ));
        let mut wire = Zeroizing::new(Vec::with_capacity(prefix.len() + body.len()));
        wire.extend_from_slice(prefix.as_bytes());
        wire.extend_from_slice(&body);

        let mut stream = TcpStream::connect(self.endpoint.address)
            .await
            .map_err(|_| AuthorizationError::Unavailable)?;
        stream
            .write_all(&wire)
            .await
            .map_err(|_| AuthorizationError::Unavailable)?;
        read_response(&mut stream).await
    }
}

#[derive(Serialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
enum AuthorizeRequest<'a> {
    Admission {
        connection_id: Uuid,
        username: &'a str,
        password: &'a str,
        target_host: &'a str,
        target_port: u16,
        #[serde(skip_serializing_if = "TargetProtocol::is_ssh")]
        target_protocol: TargetProtocol,
    },
    Continuation {
        connection_id: Uuid,
    },
}

#[derive(Serialize)]
struct CloseRequest {
    connection_id: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizeResponse {
    authorized: bool,
    lease_expires_at: DateTime<Utc>,
}

async fn read_response(stream: &mut TcpStream) -> Result<Vec<u8>, AuthorizationError> {
    let mut response = Vec::with_capacity(1_024);
    let header_end = loop {
        if let Some(position) = response.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
        if response.len() >= MAX_RESPONSE_HEADER_BYTES {
            return Err(AuthorizationError::Unavailable);
        }
        read_chunk(stream, &mut response, MAX_RESPONSE_HEADER_BYTES).await?;
    };

    let (status, content_length) = response_metadata(&response[..header_end])?;
    if matches!(status, 401 | 403) {
        return Err(AuthorizationError::Denied);
    }
    let content_length = match (status, content_length) {
        (204, None) => 0,
        (_, Some(length)) => length,
        _ => return Err(AuthorizationError::Unavailable),
    };
    if content_length > MAX_RESPONSE_BODY_BYTES {
        return Err(AuthorizationError::Unavailable);
    }
    let total = header_end
        .checked_add(content_length)
        .ok_or(AuthorizationError::Unavailable)?;
    if response.len() > total {
        return Err(AuthorizationError::Unavailable);
    }
    while response.len() < total {
        read_chunk(stream, &mut response, total).await?;
    }
    Ok(response)
}

async fn read_chunk(
    stream: &mut TcpStream,
    response: &mut Vec<u8>,
    maximum: usize,
) -> Result<(), AuthorizationError> {
    let remaining = maximum.saturating_sub(response.len());
    if remaining == 0 {
        return Err(AuthorizationError::Unavailable);
    }
    let mut chunk = [0_u8; 1_024];
    let read_limit = remaining.min(chunk.len());
    let read = stream
        .read(&mut chunk[..read_limit])
        .await
        .map_err(|_| AuthorizationError::Unavailable)?;
    if read == 0 {
        return Err(AuthorizationError::Unavailable);
    }
    response.extend_from_slice(&chunk[..read]);
    Ok(())
}

fn response_metadata(bytes: &[u8]) -> Result<(u16, Option<usize>), AuthorizationError> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut response = httparse::Response::new(&mut headers);
    let status = response
        .parse(bytes)
        .map_err(|_| AuthorizationError::Unavailable)?;
    if !status.is_complete() || response.version != Some(1) {
        return Err(AuthorizationError::Unavailable);
    }
    if response
        .headers
        .iter()
        .any(|header| header.name.eq_ignore_ascii_case("transfer-encoding"))
    {
        return Err(AuthorizationError::Unavailable);
    }
    let lengths = response
        .headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("content-length"))
        .collect::<Vec<_>>();
    if lengths.len() > 1 {
        return Err(AuthorizationError::Unavailable);
    }
    let content_length = lengths
        .first()
        .map(|header| {
            std::str::from_utf8(header.value)
                .map_err(|_| AuthorizationError::Unavailable)?
                .parse::<usize>()
                .map_err(|_| AuthorizationError::Unavailable)
        })
        .transpose()?;
    Ok((
        response.code.ok_or(AuthorizationError::Unavailable)?,
        content_length,
    ))
}

fn parse_authorization_response(
    bytes: &[u8],
    now: DateTime<Utc>,
) -> Result<Authorization, AuthorizationError> {
    let header_end = header_end(bytes)?;
    let (status, content_length) = response_metadata(&bytes[..header_end])?;
    if matches!(status, 401 | 403) {
        return Err(AuthorizationError::Denied);
    }
    if status != 200 {
        return Err(AuthorizationError::Unavailable);
    }
    let content_length = content_length.ok_or(AuthorizationError::Unavailable)?;
    if content_length > MAX_RESPONSE_BODY_BYTES || bytes.len() != header_end + content_length {
        return Err(AuthorizationError::Unavailable);
    }
    let response: AuthorizeResponse = serde_json::from_slice(&bytes[header_end..])
        .map_err(|_| AuthorizationError::Unavailable)?;
    let maximum_expiry = now + TimeDelta::seconds(MAX_AUTHORIZATION_LEASE_SECONDS);
    if !response.authorized
        || response.lease_expires_at <= now
        || response.lease_expires_at > maximum_expiry
    {
        return Err(AuthorizationError::Denied);
    }
    Ok(Authorization {
        lease_expires_at: response.lease_expires_at,
    })
}

fn parse_close_response(bytes: &[u8]) -> Result<(), AuthorizationError> {
    let header_end = header_end(bytes)?;
    let (status, content_length) = response_metadata(&bytes[..header_end])?;
    if matches!(status, 401 | 403) {
        return Err(AuthorizationError::Denied);
    }
    if status != 204 || content_length.unwrap_or_default() != 0 || bytes.len() != header_end {
        return Err(AuthorizationError::Unavailable);
    }
    Ok(())
}

fn header_end(bytes: &[u8]) -> Result<usize, AuthorizationError> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|position| position + 4)
        .ok_or(AuthorizationError::Unavailable)
}

#[cfg(test)]
mod tests {
    use crate::connect_request::TargetProtocol;
    use chrono::{TimeDelta, Utc};
    use serde_json::Value;
    use uuid::Uuid;

    use super::{
        AuthorizationError, AuthorizeRequest, parse_authorization_response, parse_close_response,
    };

    fn response(status: u16, body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    #[test]
    fn phase_requests_have_exact_frozen_shape() {
        let connection_id = Uuid::nil();
        let admission = serde_json::to_value(AuthorizeRequest::Admission {
            connection_id,
            username: "user",
            password: "secret",
            target_host: "ssh.example.com",
            target_port: 22,
            target_protocol: TargetProtocol::Ssh,
        })
        .expect("admission 应可序列化");
        assert_eq!(admission["phase"], "admission");
        assert_eq!(admission.as_object().expect("应为对象").len(), 6);
        assert!(admission.get("target_protocol").is_none());
        let rdp = serde_json::to_value(AuthorizeRequest::Admission {
            connection_id,
            username: "user",
            password: "secret",
            target_host: "rdp.example.com",
            target_port: 3389,
            target_protocol: TargetProtocol::Rdp,
        })
        .expect("RDP admission 应可序列化");
        assert_eq!(rdp["target_protocol"], "rdp");
        assert_eq!(rdp.as_object().expect("应为对象").len(), 7);

        let continuation = serde_json::to_value(AuthorizeRequest::Continuation { connection_id })
            .expect("continuation 应可序列化");
        assert_eq!(continuation["phase"], "continuation");
        assert_eq!(continuation.as_object().expect("应为对象").len(), 2);
        assert_eq!(continuation.get("password"), None::<&Value>);
        assert_eq!(continuation.get("target_host"), None::<&Value>);
    }

    #[test]
    fn accepts_only_explicit_short_lease() {
        let now = Utc::now();
        let body = serde_json::json!({
            "authorized": true,
            "lease_expires_at": now + TimeDelta::seconds(10),
        })
        .to_string();
        let result =
            parse_authorization_response(&response(200, &body), now).expect("明确短 lease 应有效");
        assert!(result.lease_expires_at > now);
    }

    #[test]
    fn fails_closed_for_non_allow_unknown_or_oversized_lease() {
        let now = Utc::now();
        let denied = serde_json::json!({
            "authorized": false,
            "lease_expires_at": now + TimeDelta::seconds(10),
        })
        .to_string();
        let unknown = serde_json::json!({
            "authorized": true,
            "lease_expires_at": now + TimeDelta::seconds(10),
            "credential": "forbidden",
        })
        .to_string();
        let too_long = serde_json::json!({
            "authorized": true,
            "lease_expires_at": now + TimeDelta::seconds(16),
        })
        .to_string();
        for raw in [
            response(500, "{}"),
            response(200, "not-json"),
            response(200, &denied),
            response(200, &unknown),
            response(200, &too_long),
        ] {
            assert!(parse_authorization_response(&raw, now).is_err());
        }
        assert!(matches!(
            parse_authorization_response(&response(401, "{}"), now),
            Err(AuthorizationError::Denied)
        ));
    }

    #[test]
    fn close_accepts_only_empty_success() {
        assert!(parse_close_response(b"HTTP/1.1 204 No Content\r\n\r\n").is_ok());
        assert!(parse_close_response(&response(200, "")).is_err());
        assert!(parse_close_response(&response(200, "{}")).is_err());
    }
}
