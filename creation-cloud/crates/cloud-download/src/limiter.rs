//! 对公开和账号下载授权实施按来源固定窗口与进程并发保护。
//! 来源 UUID 才是分桶键，内存不会保存地址、Cookie 或用户代理。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cloud_domain::{AppError, AppResult};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

const MAX_CONCURRENT_AUTHORIZATIONS: usize = 128;
const MAX_ATTEMPTS_PER_WINDOW: u16 = 600;
const MAX_BUCKETS: usize = 4_096;
const WINDOW: Duration = Duration::from_secs(60);
const OVERFLOW_BUCKET: Uuid = Uuid::from_u128(u128::MAX);

#[derive(Clone)]
pub(crate) struct DownloadLimiter {
    inner: Arc<Inner>,
}

struct Inner {
    concurrent: Arc<Semaphore>,
    attempts: Mutex<HashMap<Uuid, AttemptWindow>>,
}

#[derive(Clone, Copy)]
struct AttemptWindow {
    started_at: Instant,
    attempts: u16,
}

pub(crate) struct DownloadPermit {
    _concurrent: OwnedSemaphorePermit,
}

impl Default for DownloadLimiter {
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner {
                concurrent: Arc::new(Semaphore::new(MAX_CONCURRENT_AUTHORIZATIONS)),
                attempts: Mutex::new(HashMap::new()),
            }),
        }
    }
}

impl DownloadLimiter {
    pub(crate) fn acquire(&self, source_id: Uuid) -> AppResult<DownloadPermit> {
        let now = Instant::now();
        self.charge_at(source_id, now)?;
        let permit = self
            .inner
            .concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| rate_limited(1))?;
        Ok(DownloadPermit {
            _concurrent: permit,
        })
    }

    fn charge_at(&self, source_id: Uuid, now: Instant) -> AppResult<()> {
        let mut attempts = self
            .inner
            .attempts
            .lock()
            .map_err(|_| AppError::Internal("下载限速状态不可用".to_owned()))?;
        if attempts.len() >= MAX_BUCKETS - 1 && !attempts.contains_key(&source_id) {
            attempts.retain(|_, window| now.duration_since(window.started_at) < WINDOW);
        }
        let key = if attempts.contains_key(&source_id) || attempts.len() < MAX_BUCKETS - 1 {
            source_id
        } else {
            // 任意新来源在容量耗尽后共享溢出桶，避免错误配置导致内存无界增长。
            OVERFLOW_BUCKET
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

fn rate_limited(seconds: u64) -> AppError {
    AppError::RateLimitedAfter {
        message: "下载请求过于频繁，请稍后重试".to_owned(),
        retry_after_seconds: seconds.max(1),
    }
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
    fn limits_each_source_and_reports_retry_after() {
        let limiter = DownloadLimiter::default();
        let source = Uuid::now_v7();
        let now = Instant::now();
        for _ in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter.charge_at(source, now).expect("窗口额度内应放行");
        }
        assert!(matches!(
            limiter.charge_at(source, now),
            Err(AppError::RateLimitedAfter {
                retry_after_seconds: 1..,
                ..
            })
        ));
        limiter
            .charge_at(source, now + WINDOW)
            .expect("新窗口应恢复");
    }

    #[test]
    fn sources_have_independent_buckets() {
        let limiter = DownloadLimiter::default();
        let now = Instant::now();
        let first = Uuid::now_v7();
        for _ in 0..MAX_ATTEMPTS_PER_WINDOW {
            limiter.charge_at(first, now).expect("窗口额度内应放行");
        }
        limiter
            .charge_at(Uuid::now_v7(), now)
            .expect("另一来源应有独立额度");
    }
}
