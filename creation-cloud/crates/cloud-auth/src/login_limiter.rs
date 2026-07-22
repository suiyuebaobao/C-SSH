//! 对登录尝试实施有界并发和按标识符散列分桶的固定窗口限速。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::validation;

const MAX_CONCURRENT_ATTEMPTS: usize = 8;
const MAX_ATTEMPTS_PER_WINDOW: u16 = 10;
const MAX_BUCKETS: usize = 2_048;
const WINDOW: Duration = Duration::from_secs(60);
const INVALID_IDENTIFIER: &str = "invalid-identifier";
const OVERFLOW_BUCKET: &str = "overflow-bucket";

#[derive(Clone)]
pub(crate) struct LoginLimiter {
    inner: Arc<Inner>,
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

pub(crate) struct LoginPermit {
    _concurrent: OwnedSemaphorePermit,
}

impl Default for LoginLimiter {
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner {
                concurrent: Arc::new(Semaphore::new(MAX_CONCURRENT_ATTEMPTS)),
                attempts: Mutex::new(HashMap::new()),
            }),
        }
    }
}

impl LoginLimiter {
    pub(crate) fn acquire(&self, raw_identifier: &str) -> AppResult<LoginPermit> {
        self.charge_at(raw_identifier, Instant::now())?;
        let permit = self
            .inner
            .concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| rate_limited(1))?;
        Ok(LoginPermit {
            _concurrent: permit,
        })
    }

    fn charge_at(&self, raw_identifier: &str, now: Instant) -> AppResult<()> {
        let requested_key = identifier_key(raw_identifier);
        let overflow_key = hash_key(OVERFLOW_BUCKET);
        let mut attempts = self
            .inner
            .attempts
            .lock()
            .map_err(|_| AppError::Internal("登录限速状态不可用".to_owned()))?;

        if attempts.len() >= MAX_BUCKETS - 1 && !attempts.contains_key(&requested_key) {
            attempts.retain(|_, window| now.duration_since(window.started_at) < WINDOW);
        }
        let key = if attempts.contains_key(&requested_key) || attempts.len() < MAX_BUCKETS - 1 {
            requested_key
        } else {
            // 桶数量到达上限后，新标识符共享溢出桶，避免攻击者用随机账号撑大内存。
            overflow_key
        };
        let window = attempts.entry(key).or_insert(AttemptWindow {
            started_at: now,
            attempts: 0,
        });
        if now.duration_since(window.started_at) >= WINDOW {
            *window = AttemptWindow {
                started_at: now,
                attempts: 0,
            };
        }
        if window.attempts >= MAX_ATTEMPTS_PER_WINDOW {
            let remaining = WINDOW.saturating_sub(now.duration_since(window.started_at));
            return Err(rate_limited(retry_after_seconds(remaining)));
        }
        window.attempts += 1;
        Ok(())
    }
}

fn identifier_key(raw_identifier: &str) -> [u8; 32] {
    let canonical = if raw_identifier.len() <= 254 {
        validation::login_identifier(raw_identifier)
            .map(|identifier| identifier.value)
            .unwrap_or_else(|_| INVALID_IDENTIFIER.to_owned())
    } else {
        INVALID_IDENTIFIER.to_owned()
    };
    hash_key(&canonical)
}

fn hash_key(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}

fn retry_after_seconds(remaining: Duration) -> u64 {
    remaining
        .as_secs()
        .saturating_add(u64::from(remaining.subsec_nanos() > 0))
        .max(1)
}

fn rate_limited(seconds: u64) -> AppError {
    AppError::RateLimitedAfter {
        message: "登录尝试过于频繁，请稍后重试".to_owned(),
        retry_after_seconds: seconds.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_each_normalized_identifier_and_resets_after_window() {
        let limiter = LoginLimiter::default();
        let now = Instant::now();
        for _ in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter
                .charge_at(" Admin_User ", now)
                .expect("窗口内前十次应放行");
        }
        assert!(matches!(
            limiter.charge_at("admin_user", now),
            Err(AppError::RateLimitedAfter { .. })
        ));
        limiter
            .charge_at("admin_user", now + WINDOW)
            .expect("新窗口应重新放行");
    }

    #[test]
    fn invalid_identifiers_share_one_bounded_bucket() {
        let limiter = LoginLimiter::default();
        let now = Instant::now();
        for index in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter
                .charge_at(&format!("invalid.{index}"), now)
                .expect("非法标识符共享桶的窗口额度应有界");
        }
        assert!(matches!(
            limiter.charge_at("another.invalid", now),
            Err(AppError::RateLimitedAfter { .. })
        ));
    }

    #[test]
    fn rejects_more_than_eight_concurrent_attempts() {
        let limiter = LoginLimiter::default();
        let mut permits = Vec::new();
        for index in 0..MAX_CONCURRENT_ATTEMPTS {
            permits.push(
                limiter
                    .acquire(&format!("user-{index}@example.com"))
                    .expect("并发闸门容量内应放行"),
            );
        }
        assert!(matches!(
            limiter.acquire("overflow@example.com"),
            Err(AppError::RateLimitedAfter { .. })
        ));
        permits.pop();
        limiter
            .acquire("released@example.com")
            .expect("释放许可后应重新放行");
    }
}
