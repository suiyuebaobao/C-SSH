//! 生成六位验证码并以 HMAC-SHA256 形成不可逆数据库校验值。

use rand::Rng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

pub(crate) const CODE_TTL_MINUTES: i64 = 10;
pub(crate) const MAX_ATTEMPTS: i32 = 5;
pub(crate) const RESEND_COOLDOWN_SECONDS: i64 = 60;

pub(crate) fn issue_code() -> String {
    format!("{:06}", rand::rng().random_range(0_u32..1_000_000))
}

pub(crate) fn digest(key: &[u8], challenge_id: Uuid, email: &str, code: &str) -> [u8; 32] {
    let mut message = Vec::with_capacity(16 + email.len() + code.len() + 2);
    message.extend_from_slice(challenge_id.as_bytes());
    message.push(0);
    message.extend_from_slice(email.as_bytes());
    message.push(0);
    message.extend_from_slice(code.as_bytes());
    hmac_sha256(key, &message)
}

pub(crate) fn matches(expected: &[u8], actual: &[u8; 32]) -> bool {
    expected.len() == actual.len() && bool::from(expected.ct_eq(actual))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_BYTES: usize = 64;
    let mut normalized = [0_u8; BLOCK_BYTES];
    if key.len() > BLOCK_BYTES {
        normalized[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36_u8; BLOCK_BYTES];
    let mut outer_pad = [0x5c_u8; BLOCK_BYTES];
    for index in 0..BLOCK_BYTES {
        inner_pad[index] ^= normalized[index];
        outer_pad[index] ^= normalized[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_code_is_always_six_ascii_digits() {
        for _ in 0..100 {
            let code = issue_code();
            assert_eq!(code.len(), 6);
            assert!(code.bytes().all(|byte| byte.is_ascii_digit()));
        }
    }

    #[test]
    fn hmac_matches_the_rfc_4231_sha256_vector() {
        let digest = hmac_sha256(&[0x0b; 20], b"Hi There");
        assert_eq!(
            hex::encode(digest),
            "b0344c61d8db38535ca8afceaf0bf12b\
             881dc200c9833da726e9376c2e32cff7"
                .replace(' ', "")
        );
    }

    #[test]
    fn digest_is_bound_to_email_challenge_and_code() {
        let id = Uuid::now_v7();
        let expected = digest(b"test-key", id, "user@example.com", "123456");
        assert!(matches(
            &expected,
            &digest(b"test-key", id, "user@example.com", "123456")
        ));
        assert!(!matches(
            &expected,
            &digest(b"test-key", id, "other@example.com", "123456")
        ));
    }
}
