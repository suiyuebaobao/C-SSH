//! 为同步 API 提供进程内、账号级的固定窗口限速与并发闸门。
//! 账号桶数量严格有界，容量耗尽时新账号共享溢出桶，许可随作用域自动释放。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cloud_domain::{AppError, AppResult};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

const WINDOW: Duration = Duration::from_secs(60);
const MAX_WRITE_REQUESTS: u16 = 60;
const MAX_READ_REQUESTS: u16 = 300;
const MAX_ACCOUNT_CONCURRENCY: usize = 8;
const MAX_PROCESS_CONCURRENCY: usize = 64;
const MAX_ACCOUNT_BUCKETS: usize = 4_096;

#[derive(Clone, Copy, Debug)]
pub(crate) enum AccessKind {
    Read,
    Write,
}

impl AccessKind {
    const fn limit(self) -> u16 {
        match self {
            Self::Read => MAX_READ_REQUESTS,
            Self::Write => MAX_WRITE_REQUESTS,
        }
    }
}

#[derive(Clone)]
pub(crate) struct SyncLimiter {
    inner: Arc<Inner>,
}

struct Inner {
    global: Arc<Semaphore>,
    buckets: Mutex<HashMap<Uuid, Arc<AccountBucket>>>,
    overflow: Arc<AccountBucket>,
}

struct AccountBucket {
    concurrent: Arc<Semaphore>,
    window: Mutex<UsageWindow>,
}

#[derive(Clone, Copy)]
struct UsageWindow {
    started_at: Instant,
    reads: u16,
    writes: u16,
}

#[derive(Debug)]
pub(crate) struct SyncPermit {
    _global: OwnedSemaphorePermit,
    _account: OwnedSemaphorePermit,
}

impl Default for SyncLimiter {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            inner: Arc::new(Inner {
                global: Arc::new(Semaphore::new(MAX_PROCESS_CONCURRENCY)),
                buckets: Mutex::new(HashMap::new()),
                overflow: Arc::new(AccountBucket::new(now)),
            }),
        }
    }
}

impl SyncLimiter {
    pub(crate) fn acquire(&self, account_id: Uuid, kind: AccessKind) -> AppResult<SyncPermit> {
        self.acquire_at_inner(account_id, kind, Instant::now())
    }

    fn acquire_at_inner(
        &self,
        account_id: Uuid,
        kind: AccessKind,
        now: Instant,
    ) -> AppResult<SyncPermit> {
        let bucket = self.bucket_for(account_id, now)?;
        // 请求先计入固定窗口；即使随后撞到并发闸门也消耗额度，防止高频探测闸门绕过限速。
        bucket.charge(kind, now)?;

        let global = self
            .inner
            .global
            .clone()
            .try_acquire_owned()
            .map_err(|_| concurrent_limited())?;
        let account = bucket
            .concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| concurrent_limited())?;
        Ok(SyncPermit {
            _global: global,
            _account: account,
        })
    }

    fn bucket_for(&self, account_id: Uuid, now: Instant) -> AppResult<Arc<AccountBucket>> {
        let mut buckets = self
            .inner
            .buckets
            .lock()
            .map_err(|_| AppError::Internal("同步限流账号桶不可用".to_owned()))?;
        if let Some(bucket) = buckets.get(&account_id) {
            return Ok(Arc::clone(bucket));
        }
        if buckets.len() >= MAX_ACCOUNT_BUCKETS {
            // 仅在容量耗尽时做一次有界扫描，不在常规请求路径延长全局账号桶锁。
            buckets.retain(|_, bucket| !can_evict(bucket, now));
            if buckets.len() >= MAX_ACCOUNT_BUCKETS {
                return Ok(Arc::clone(&self.inner.overflow));
            }
        }
        let bucket = Arc::new(AccountBucket::new(now));
        buckets.insert(account_id, Arc::clone(&bucket));
        Ok(bucket)
    }

    #[cfg(test)]
    pub(crate) fn acquire_at(
        &self,
        account_id: Uuid,
        kind: AccessKind,
        now: Instant,
    ) -> AppResult<SyncPermit> {
        self.acquire_at_inner(account_id, kind, now)
    }

    #[cfg(test)]
    pub(crate) fn bucket_count(&self) -> usize {
        self.inner
            .buckets
            .lock()
            .expect("测试中的同步限流账号桶不应中毒")
            .len()
    }

    #[cfg(test)]
    pub(crate) fn uses_overflow_at(&self, account_id: Uuid, now: Instant) -> bool {
        let bucket = self
            .bucket_for(account_id, now)
            .expect("测试中的同步限流账号桶应可取得");
        Arc::ptr_eq(&bucket, &self.inner.overflow)
    }
}

fn can_evict(bucket: &Arc<AccountBucket>, now: Instant) -> bool {
    if Arc::strong_count(bucket) != 1
        || bucket.concurrent.available_permits() != MAX_ACCOUNT_CONCURRENCY
    {
        return false;
    }
    bucket
        .window
        .lock()
        .is_ok_and(|window| now.saturating_duration_since(window.started_at) >= WINDOW)
}

impl AccountBucket {
    fn new(now: Instant) -> Self {
        Self {
            concurrent: Arc::new(Semaphore::new(MAX_ACCOUNT_CONCURRENCY)),
            window: Mutex::new(UsageWindow {
                started_at: now,
                reads: 0,
                writes: 0,
            }),
        }
    }

    fn charge(&self, kind: AccessKind, now: Instant) -> AppResult<()> {
        let mut window = self
            .window
            .lock()
            .map_err(|_| AppError::Internal("同步限流计数器不可用".to_owned()))?;
        let elapsed = now.saturating_duration_since(window.started_at);
        if elapsed >= WINDOW {
            *window = UsageWindow {
                started_at: now,
                reads: 0,
                writes: 0,
            };
        }
        let count = match kind {
            AccessKind::Read => &mut window.reads,
            AccessKind::Write => &mut window.writes,
        };
        if *count >= kind.limit() {
            let remaining = WINDOW.saturating_sub(now.saturating_duration_since(window.started_at));
            return Err(window_limited(retry_after_seconds(remaining)));
        }
        *count += 1;
        Ok(())
    }
}

fn retry_after_seconds(remaining: Duration) -> u64 {
    remaining
        .as_secs()
        .saturating_add(u64::from(remaining.subsec_nanos() > 0))
        .max(1)
}

fn window_limited(retry_after_seconds: u64) -> AppError {
    AppError::RateLimitedAfter {
        message: "同步请求过于频繁，请稍后重试".to_owned(),
        retry_after_seconds,
    }
}

fn concurrent_limited() -> AppError {
    AppError::RateLimitedAfter {
        message: "同步并发请求过多，请稍后重试".to_owned(),
        retry_after_seconds: 1,
    }
}

#[cfg(test)]
pub(crate) const TEST_MAX_ACCOUNT_BUCKETS: usize = MAX_ACCOUNT_BUCKETS;
