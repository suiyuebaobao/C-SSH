//! 对注册与修改密码实施相互独立的账号级固定窗口和进程并发保护。
//! 内存只保存规范化标识的 SHA256，不保留邮箱、账号或密码正文。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

use crate::validation;

const MAX_CONCURRENT_OPERATIONS: usize = 4;
const MAX_ATTEMPTS_PER_WINDOW: u16 = 5;
const MAX_BUCKETS: usize = 2_048;
const WINDOW: Duration = Duration::from_secs(60);
const INVALID_EMAIL: &str = "invalid-email";
const OVERFLOW_BUCKET: &str = "overflow-bucket";

#[derive(Clone)]
pub(crate) struct CredentialLimiter {
    register: FixedWindowLimiter,
    password: FixedWindowLimiter,
}

#[derive(Clone)]
struct FixedWindowLimiter {
    inner: Arc<Inner>,
    message: &'static str,
}

struct Inner {
    concurrent: Arc<Semaphore>,
    attempts: Mutex<HashMap<[u8; 32], AttemptWindow>>,
}

#[derive(Clone, Copy)]
struct AttemptWindow {
    started_at: Instant,
    attempts: u16,
}

pub(crate) struct CredentialPermit {
    _concurrent: OwnedSemaphorePermit,
}

impl Default for CredentialLimiter {
    fn default() -> Self {
        Self {
            register: FixedWindowLimiter::new("注册请求过于频繁，请稍后重试"),
            password: FixedWindowLimiter::new("密码操作过于频繁，请稍后重试"),
        }
    }
}

impl CredentialLimiter {
    pub(crate) fn acquire_register(&self, raw_email: &str) -> AppResult<CredentialPermit> {
        let canonical = if raw_email.len() <= 254 {
            validation::normalize_email(raw_email).unwrap_or_else(|_| INVALID_EMAIL.to_owned())
        } else {
            INVALID_EMAIL.to_owned()
        };
        self.register.acquire(canonical.as_bytes())
    }

    pub(crate) fn acquire_password(&self, account_id: Uuid) -> AppResult<CredentialPermit> {
        self.password.acquire(account_id.as_bytes())
    }
}

impl FixedWindowLimiter {
    fn new(message: &'static str) -> Self {
        Self {
            inner: Arc::new(Inner {
                concurrent: Arc::new(Semaphore::new(MAX_CONCURRENT_OPERATIONS)),
                attempts: Mutex::new(HashMap::new()),
            }),
            message,
        }
    }

    fn acquire(&self, canonical_key: &[u8]) -> AppResult<CredentialPermit> {
        let now = Instant::now();
        self.charge_at(canonical_key, now)?;
        let permit = self
            .inner
            .concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| self.rate_limited(1))?;
        Ok(CredentialPermit {
            _concurrent: permit,
        })
    }

    fn charge_at(&self, canonical_key: &[u8], now: Instant) -> AppResult<()> {
        let requested_key = hash_key(canonical_key);
        let overflow_key = hash_key(OVERFLOW_BUCKET.as_bytes());
        let mut attempts = self
            .inner
            .attempts
            .lock()
            .map_err(|_| AppError::Internal("凭据操作限速状态不可用".to_owned()))?;
        if attempts.len() >= MAX_BUCKETS - 1 && !attempts.contains_key(&requested_key) {
            attempts.retain(|_, window| now.duration_since(window.started_at) < WINDOW);
        }
        let key = if attempts.contains_key(&requested_key) || attempts.len() < MAX_BUCKETS - 1 {
            requested_key
        } else {
            // 容量耗尽后的新标识共享溢出桶，随机邮箱不能让内存无界增长。
            overflow_key
        };
        let window = attempts.entry(key).or_insert(AttemptWindow {
            started_at: now,
            attempts: 0,
        });
        let elapsed = now.duration_since(window.started_at);
        if elapsed >= WINDOW {
            *window = AttemptWindow {
                started_at: now,
                attempts: 0,
            };
        }
        if window.attempts >= MAX_ATTEMPTS_PER_WINDOW {
            let remaining = WINDOW.saturating_sub(now.duration_since(window.started_at));
            return Err(self.rate_limited(retry_after_seconds(remaining)));
        }
        window.attempts += 1;
        Ok(())
    }

    fn rate_limited(&self, seconds: u64) -> AppError {
        AppError::RateLimitedAfter {
            message: self.message.to_owned(),
            retry_after_seconds: seconds.max(1),
        }
    }
}

fn hash_key(value: &[u8]) -> [u8; 32] {
    Sha256::digest(value).into()
}

fn retry_after_seconds(remaining: Duration) -> u64 {
    remaining
        .as_secs()
        .saturating_add(u64::from(remaining.subsec_nanos() > 0))
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_normalizes_email_and_returns_retry_after() {
        let limiter = CredentialLimiter::default();
        for _ in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter
                .acquire_register(" User@Example.COM ")
                .expect("窗口额度内应放行");
        }
        assert!(matches!(
            limiter.acquire_register("user@example.com"),
            Err(AppError::RateLimitedAfter {
                retry_after_seconds: 1..,
                ..
            })
        ));
    }

    #[test]
    fn password_operations_are_isolated_by_account() {
        let limiter = CredentialLimiter::default();
        let first = Uuid::now_v7();
        let second = Uuid::now_v7();
        for _ in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter
                .acquire_password(first)
                .expect("同账号窗口额度内应放行");
        }
        assert!(limiter.acquire_password(first).is_err());
        limiter
            .acquire_password(second)
            .expect("另一账号应有独立额度");
    }

    #[test]
    fn each_operation_has_four_concurrent_permits() {
        let limiter = CredentialLimiter::default();
        let mut permits = Vec::new();
        for index in 0..MAX_CONCURRENT_OPERATIONS {
            permits.push(
                limiter
                    .acquire_register(&format!("user-{index}@example.com"))
                    .expect("并发容量内应放行"),
            );
        }
        assert!(matches!(
            limiter.acquire_register("overflow@example.com"),
            Err(AppError::RateLimitedAfter { .. })
        ));
    }
}
