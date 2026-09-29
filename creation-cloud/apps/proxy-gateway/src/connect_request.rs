//! 严格解析单个 HTTP/1.1 CONNECT 请求和 Basic 短凭据。

use std::{net::IpAddr, str::FromStr};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use http::uri::Authority;
use serde::Serialize;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use zeroize::{Zeroize, Zeroizing};

const MAX_HEADER_BYTES: usize = 8 * 1024;
const MAX_HEADERS: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TargetProtocol {
    #[default]
    Ssh,
    Rdp,
}

impl TargetProtocol {
    pub(crate) fn is_ssh(&self) -> bool {
        *self == Self::Ssh
    }
}

#[derive(Debug, Error)]
pub(crate) enum ConnectRequestError {
    #[error("proxy credentials are missing or invalid")]
    Credentials,
    #[error("CONNECT request is invalid")]
    Invalid,
}

pub(crate) struct Credentials {
    pub username: Zeroizing<String>,
    pub password: Zeroizing<String>,
}

pub(crate) struct ConnectRequest {
    pub host: String,
    pub port: u16,
    pub protocol: TargetProtocol,
    pub credentials: Credentials,
    pub prefetched: Zeroizing<Vec<u8>>,
}

pub(crate) async fn read_from<S>(stream: &mut S) -> Result<ConnectRequest, ConnectRequestError>
where
    S: AsyncRead + Unpin,
{
    let mut buffer = Zeroizing::new(Vec::with_capacity(1_024));
    let header_end = loop {
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
        if buffer.len() >= MAX_HEADER_BYTES {
            return Err(ConnectRequestError::Invalid);
        }
        let mut chunk = [0_u8; 1_024];
        let read = stream
            .read(&mut chunk)
            .await
            .map_err(|_| ConnectRequestError::Invalid)?;
        if read == 0 || buffer.len().saturating_add(read) > MAX_HEADER_BYTES {
            chunk.zeroize();
            return Err(ConnectRequestError::Invalid);
        }
        buffer.extend_from_slice(&chunk[..read]);
        chunk.zeroize();
    };

    let mut request = parse_header(&buffer[..header_end])?;
    let prefetched = buffer.split_off(header_end);
    buffer.zeroize();
    request.prefetched = Zeroizing::new(prefetched);
    Ok(request)
}

fn parse_header(bytes: &[u8]) -> Result<ConnectRequest, ConnectRequestError> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut parsed = httparse::Request::new(&mut headers);
    let status = parsed
        .parse(bytes)
        .map_err(|_| ConnectRequestError::Invalid)?;
    if !status.is_complete() || parsed.method != Some("CONNECT") || parsed.version != Some(1) {
        return Err(ConnectRequestError::Invalid);
    }
    let target = parsed.path.ok_or(ConnectRequestError::Invalid)?;
    let (host, port) = parse_authority(target)?;

    let host_header = exactly_one(parsed.headers, "host")?;
    let (header_host, header_port) = parse_authority(header_host_value(host_header)?)?;
    if host != header_host || port != header_port {
        return Err(ConnectRequestError::Invalid);
    }
    reject_body_headers(parsed.headers)?;
    let protocol = target_protocol(parsed.headers)?;

    let authorization = exactly_one(parsed.headers, "proxy-authorization")
        .map_err(|_| ConnectRequestError::Credentials)?;
    let credentials = parse_basic(authorization)?;
    Ok(ConnectRequest {
        host,
        port,
        protocol,
        credentials,
        prefetched: Zeroizing::new(Vec::new()),
    })
}

fn target_protocol(
    headers: &[httparse::Header<'_>],
) -> Result<TargetProtocol, ConnectRequestError> {
    let mut values = headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("x-cssh-target-protocol"));
    let first = values.next();
    if values.next().is_some() {
        return Err(ConnectRequestError::Invalid);
    }
    match first.map(|header| header.value) {
        None | Some(b"ssh") => Ok(TargetProtocol::Ssh),
        Some(b"rdp") => Ok(TargetProtocol::Rdp),
        _ => Err(ConnectRequestError::Invalid),
    }
}

fn exactly_one<'a>(
    headers: &'a [httparse::Header<'a>],
    name: &str,
) -> Result<&'a [u8], ConnectRequestError> {
    let mut matches = headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case(name));
    let value = matches
        .next()
        .map(|header| header.value)
        .ok_or(ConnectRequestError::Invalid)?;
    if matches.next().is_some() {
        return Err(ConnectRequestError::Invalid);
    }
    Ok(value)
}

fn header_host_value(value: &[u8]) -> Result<&str, ConnectRequestError> {
    let value = std::str::from_utf8(value).map_err(|_| ConnectRequestError::Invalid)?;
    if value.is_empty() || value.trim_matches([' ', '\t']) != value {
        return Err(ConnectRequestError::Invalid);
    }
    Ok(value)
}

fn reject_body_headers(headers: &[httparse::Header<'_>]) -> Result<(), ConnectRequestError> {
    if headers
        .iter()
        .any(|header| header.name.eq_ignore_ascii_case("transfer-encoding"))
    {
        return Err(ConnectRequestError::Invalid);
    }
    let lengths = headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("content-length"))
        .collect::<Vec<_>>();
    if lengths.len() > 1 {
        return Err(ConnectRequestError::Invalid);
    }
    if let Some(header) = lengths.first()
        && header.value != b"0"
    {
        return Err(ConnectRequestError::Invalid);
    }
    Ok(())
}

fn parse_authority(value: &str) -> Result<(String, u16), ConnectRequestError> {
    if value.contains(['@', '/', '?', '#']) {
        return Err(ConnectRequestError::Invalid);
    }
    let authority = Authority::from_str(value).map_err(|_| ConnectRequestError::Invalid)?;
    let port = authority.port_u16().ok_or(ConnectRequestError::Invalid)?;
    if port == 0 {
        return Err(ConnectRequestError::Invalid);
    }
    let host = authority.host().trim_matches(['[', ']']);
    normalize_host(host).map(|host| (host, port))
}

fn normalize_host(host: &str) -> Result<String, ConnectRequestError> {
    if host.is_empty() || host.len() > 253 {
        return Err(ConnectRequestError::Invalid);
    }
    if let Ok(address) = IpAddr::from_str(host) {
        return Ok(address.to_string());
    }
    if !host.is_ascii() || host.ends_with('.') || !host.contains('.') {
        return Err(ConnectRequestError::Invalid);
    }
    let normalized = host.to_ascii_lowercase();
    let valid = normalized.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    });
    if !valid {
        return Err(ConnectRequestError::Invalid);
    }
    Ok(normalized)
}

fn parse_basic(value: &[u8]) -> Result<Credentials, ConnectRequestError> {
    let value = std::str::from_utf8(value).map_err(|_| ConnectRequestError::Credentials)?;
    if !value.is_ascii()
        || value.len() <= 6
        || !value[..5].eq_ignore_ascii_case("basic")
        || value.as_bytes()[5] != b' '
    {
        return Err(ConnectRequestError::Credentials);
    }
    let encoded = &value[6..];
    if encoded.is_empty() || encoded.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(ConnectRequestError::Credentials);
    }
    let decoded_bytes = Zeroizing::new(
        STANDARD
            .decode(encoded)
            .map_err(|_| ConnectRequestError::Credentials)?,
    );
    let canonical = Zeroizing::new(STANDARD.encode(decoded_bytes.as_slice()));
    if canonical.as_str() != encoded {
        return Err(ConnectRequestError::Credentials);
    }
    let decoded = Zeroizing::new(
        String::from_utf8(decoded_bytes.to_vec()).map_err(|_| ConnectRequestError::Credentials)?,
    );
    let (username, password) = decoded
        .split_once(':')
        .ok_or(ConnectRequestError::Credentials)?;
    let username_valid = username.len() == 37
        && username.starts_with("cssh_")
        && username[5..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    let password_valid = password.len() == 43
        && password
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    if !username_valid || !password_valid {
        return Err(ConnectRequestError::Credentials);
    }
    Ok(Credentials {
        username: Zeroizing::new(username.to_owned()),
        password: Zeroizing::new(password.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    use super::{ConnectRequestError, TargetProtocol, parse_header};

    fn authorization() -> String {
        let username = format!("cssh_{}", "a".repeat(32));
        let password = "B".repeat(43);
        format!(
            "Basic {}",
            STANDARD.encode(format!("{username}:{password}"))
        )
    }

    #[test]
    fn accepts_only_matching_http11_connect_with_short_credentials() {
        let raw = format!(
            "CONNECT SSH.Example.COM:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\nProxy-Authorization: {}\r\nContent-Length: 0\r\n\r\n",
            authorization()
        );
        let request = parse_header(raw.as_bytes()).expect("严格 CONNECT 应有效");
        assert_eq!(request.host, "ssh.example.com");
        assert_eq!(request.port, 22);
        assert_eq!(request.protocol, TargetProtocol::Ssh);
    }

    #[test]
    fn target_protocol_is_explicit_and_single_valued() {
        for (headers, expected) in [
            ("X-CSSH-Target-Protocol: rdp\r\n", Some(TargetProtocol::Rdp)),
            ("x-cssh-target-protocol: ssh\r\n", Some(TargetProtocol::Ssh)),
            ("X-CSSH-Target-Protocol: tcp\r\n", None),
            ("X-CSSH-Target-Protocol: RDP\r\n", None),
            ("X-CSSH-Target-Protocol: ssh,rdp\r\n", None),
            ("X-CSSH-Target-Protocol:\r\n", None),
            (
                "X-CSSH-Target-Protocol: rdp\r\nx-cssh-target-protocol: rdp\r\n",
                None,
            ),
        ] {
            let raw = format!(
                "CONNECT ssh.example.com:3389 HTTP/1.1\r\nHost: ssh.example.com:3389\r\nProxy-Authorization: {}\r\n{headers}\r\n",
                authorization()
            );
            let parsed = parse_header(raw.as_bytes());
            match expected {
                Some(expected) => assert_eq!(parsed.expect("协议头应有效").protocol, expected),
                None => assert!(matches!(parsed, Err(ConnectRequestError::Invalid))),
            }
        }
    }

    #[test]
    fn rejects_missing_duplicate_or_invalid_credentials() {
        let valid = authorization();
        for raw in [
            "CONNECT ssh.example.com:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\n\r\n".to_owned(),
            format!("CONNECT ssh.example.com:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\nProxy-Authorization: {valid}\r\nProxy-Authorization: {valid}\r\n\r\n"),
            "CONNECT ssh.example.com:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\nProxy-Authorization: Basic eA==\r\n\r\n".to_owned(),
        ] {
            assert!(matches!(
                parse_header(raw.as_bytes()),
                Err(ConnectRequestError::Credentials)
            ));
        }
    }

    #[test]
    fn rejects_method_host_mismatch_and_request_body() {
        let authorization = authorization();
        for raw in [
            format!(
                "GET ssh.example.com:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\nProxy-Authorization: {authorization}\r\n\r\n"
            ),
            format!(
                "CONNECT ssh.example.com:22 HTTP/1.1\r\nHost: other.example.com:22\r\nProxy-Authorization: {authorization}\r\n\r\n"
            ),
            format!(
                "CONNECT ssh.example.com:22 HTTP/1.1\r\nHost: ssh.example.com:22\r\nProxy-Authorization: {authorization}\r\nContent-Length: 1\r\n\r\n"
            ),
        ] {
            assert!(matches!(
                parse_header(raw.as_bytes()),
                Err(ConnectRequestError::Invalid)
            ));
        }
    }
}
