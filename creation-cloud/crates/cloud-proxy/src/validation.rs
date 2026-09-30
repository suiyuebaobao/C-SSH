//! 规范化目标并拒绝会造成目标摘要歧义的主机和端口表示。

use std::{net::IpAddr, str::FromStr};

use crate::{
    ProxyError,
    model::{ProxyTarget, TargetProtocol},
};

pub(crate) struct NormalizedTarget {
    pub host: String,
    pub port: u16,
    pub protocol: TargetProtocol,
}

pub(crate) fn target(input: &ProxyTarget) -> Result<NormalizedTarget, ProxyError> {
    normalize_target(&input.host, input.port, input.protocol)
}

pub(crate) fn authorize_target(
    host: &str,
    port: u16,
    protocol: TargetProtocol,
) -> Result<NormalizedTarget, ProxyError> {
    normalize_target(host, port, protocol).map_err(|_| ProxyError::AuthorizationDenied)
}

fn normalize_target(
    host: &str,
    port: u16,
    protocol: TargetProtocol,
) -> Result<NormalizedTarget, ProxyError> {
    if port == 0 || host.is_empty() || host.len() > 253 || host.trim() != host {
        return Err(ProxyError::InvalidRequest);
    }
    if let Ok(address) = IpAddr::from_str(host) {
        return Ok(NormalizedTarget {
            host: address.to_string(),
            port,
            protocol,
        });
    }
    if !host.is_ascii() || host.ends_with('.') || !host.contains('.') {
        return Err(ProxyError::InvalidRequest);
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
        return Err(ProxyError::InvalidRequest);
    }
    Ok(NormalizedTarget {
        host: normalized,
        port,
        protocol,
    })
}

pub(crate) fn internal_credential(username: &str, password: &str) -> Result<(), ProxyError> {
    let username_valid = username.len() == 37
        && username.starts_with("cssh_")
        && username[5..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !username_valid || password.len() != 43 || !password.is_ascii() {
        return Err(ProxyError::AuthorizationDenied);
    }
    Ok(())
}

pub(crate) fn bearer(value: &str) -> Result<&str, ProxyError> {
    let (scheme, token) = value
        .split_once(' ')
        .ok_or(ProxyError::AuthorizationDenied)?;
    if !scheme.eq_ignore_ascii_case("bearer")
        || token.len() < 32
        || token.len() > 512
        || !token.is_ascii()
        || token.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err(ProxyError::AuthorizationDenied);
    }
    Ok(token)
}
