//! 以用途域隔离的 HMAC 生成目标摘要、幂等摘要和可重建短凭据。

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{TargetProtocol, validation::NormalizedTarget};

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn target_digest(key: &[u8], target: &NormalizedTarget) -> [u8; 32] {
    // SSH 的历史用途域及字段字节保持不变，RDP 不能复用同地址的 SSH admission。
    let domain: &[u8] = match target.protocol {
        TargetProtocol::Ssh => b"creation-cloud/proxy/target/v1",
        TargetProtocol::Rdp => b"creation-cloud/proxy/target/rdp/v1",
    };
    sign(
        key,
        domain,
        &[target.host.as_bytes(), &target.port.to_be_bytes()],
    )
}

pub(crate) fn request_digest(key: &[u8], node_id: Uuid, target_digest: &[u8; 32]) -> [u8; 32] {
    sign(
        key,
        b"creation-cloud/proxy/mutation/v1",
        &[node_id.as_bytes(), target_digest],
    )
}

pub(crate) fn credential_username(credential_id: Uuid) -> String {
    format!("cssh_{}", credential_id.simple())
}

pub(crate) fn credential_password(key: &[u8], credential_id: Uuid) -> String {
    let value = sign(
        key,
        b"creation-cloud/proxy/password/v1",
        &[credential_id.as_bytes()],
    );
    URL_SAFE_NO_PAD.encode(value)
}

pub(crate) fn password_digest(key: &[u8], password: &str) -> [u8; 32] {
    sign(
        key,
        b"creation-cloud/proxy/password-digest/v1",
        &[password.as_bytes()],
    )
}

pub(crate) fn node_token_digest(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

pub(crate) fn matches(expected: &[u8], actual: &[u8; 32]) -> bool {
    expected.len() == actual.len() && expected.ct_eq(actual).into()
}

fn sign(key: &[u8], domain: &[u8], fields: &[&[u8]]) -> [u8; 32] {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC-SHA256 accepts every key length");
    mac.update(domain);
    for field in fields {
        mac.update(&(field.len() as u64).to_be_bytes());
        mac.update(field);
    }
    mac.finalize().into_bytes().into()
}
