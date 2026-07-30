//! 生成六位验证码并以 HMAC-SHA256 形成不可逆数据库校验值。

use rand::Rng;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

pub(crate) const CODE_TTL_MINUTES: i64 = 10;
pub(crate) const MAX_ATTEMPTS: i32 = 5;
pub(crate) const RESEND_COOLDOWN_SECONDS: i64 = 60;
const LOGIN_CONTEXT: &[u8] = b"creation-cloud-login-verification-v1\0";
const CAPTCHA_CONTEXT: &[u8] = b"creation-cloud-auth-captcha-v2\0";

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

pub(crate) fn login_digest(
    key: &[u8],
    challenge_id: Uuid,
    account_id: Uuid,
    email: &str,
    credential_version: i64,
    code: &str,
) -> [u8; 32] {
    let mut message =
        Vec::with_capacity(LOGIN_CONTEXT.len() + 16 + 16 + email.len() + code.len() + 11);
    message.extend_from_slice(LOGIN_CONTEXT);
    message.extend_from_slice(challenge_id.as_bytes());
    message.extend_from_slice(account_id.as_bytes());
    message.extend_from_slice(&credential_version.to_be_bytes());
    message.push(0);
    message.extend_from_slice(email.as_bytes());
    message.push(0);
    message.extend_from_slice(code.as_bytes());
    hmac_sha256(key, &message)
}

pub(crate) fn captcha_digest(
    key: &[u8],
    challenge_id: Uuid,
    purpose: crate::captcha::CaptchaPurpose,
    code: &str,
) -> [u8; 32] {
    let mut message =
        Vec::with_capacity(CAPTCHA_CONTEXT.len() + 16 + purpose.as_str().len() + code.len() + 1);
    message.extend_from_slice(CAPTCHA_CONTEXT);
    message.extend_from_slice(challenge_id.as_bytes());
    message.extend_from_slice(purpose.as_str().as_bytes());
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

    #[test]
    fn login_digest_is_domain_separated_and_bound_to_the_account_snapshot() {
        let challenge_id = Uuid::now_v7();
        let account_id = Uuid::now_v7();
        let expected = login_digest(
            b"test-key",
            challenge_id,
            account_id,
            "user@example.com",
            7,
            "123456",
        );
        assert!(matches(
            &expected,
            &login_digest(
                b"test-key",
                challenge_id,
                account_id,
                "user@example.com",
                7,
                "123456",
            )
        ));
        assert!(!matches(
            &expected,
            &login_digest(
                b"test-key",
                challenge_id,
                Uuid::now_v7(),
                "user@example.com",
                7,
                "123456",
            )
        ));
        assert!(!matches(
            &expected,
            &login_digest(
                b"test-key",
                challenge_id,
                account_id,
                "user@example.com",
                8,
                "123456",
            )
        ));
        assert_ne!(
            expected,
            digest(b"test-key", challenge_id, "user@example.com", "123456")
        );
    }

    #[test]
    fn captcha_digest_is_domain_separated_and_bound_to_the_challenge() {
        let challenge_id = Uuid::now_v7();
        let expected = captcha_digest(
            b"test-key",
            challenge_id,
            crate::captcha::CaptchaPurpose::Login,
            "123456",
        );
        assert!(matches(
            &expected,
            &captcha_digest(
                b"test-key",
                challenge_id,
                crate::captcha::CaptchaPurpose::Login,
                "123456"
            )
        ));
        assert!(!matches(
            &expected,
            &captcha_digest(
                b"test-key",
                Uuid::now_v7(),
                crate::captcha::CaptchaPurpose::Login,
                "123456"
            )
        ));
        assert!(!matches(
            &expected,
            &captcha_digest(
                b"test-key",
                challenge_id,
                crate::captcha::CaptchaPurpose::Register,
                "123456"
            )
        ));
        assert_ne!(
            expected,
            digest(b"test-key", challenge_id, "user@example.com", "123456")
        );
    }
}
