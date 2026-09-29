//! 以不进入日志的来源 IP 计数限制单来源并发，并用 RAII 精确释放额度。

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) struct SourceLimiter {
    maximum: usize,
    counts: Arc<Mutex<HashMap<IpAddr, usize>>>,
}

pub(crate) struct SourcePermit {
    source: IpAddr,
    counts: Arc<Mutex<HashMap<IpAddr, usize>>>,
}

impl SourceLimiter {
    pub(crate) fn new(maximum: usize) -> Self {
        Self {
            maximum,
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn try_acquire(&self, source: IpAddr) -> Option<SourcePermit> {
        let source = normalize(source);
        let mut counts = self.counts.lock().ok()?;
        let count = counts.entry(source).or_default();
        if *count >= self.maximum {
            return None;
        }
        *count += 1;
        Some(SourcePermit {
            source,
            counts: Arc::clone(&self.counts),
        })
    }
}

impl Drop for SourcePermit {
    fn drop(&mut self) {
        let Ok(mut counts) = self.counts.lock() else {
            return;
        };
        let Some(count) = counts.get_mut(&self.source) else {
            return;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            counts.remove(&self.source);
        }
    }
}

fn normalize(source: IpAddr) -> IpAddr {
    match source {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map_or(IpAddr::V6(address), IpAddr::V4),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::SourceLimiter;

    #[test]
    fn source_limit_restores_capacity_and_isolates_sources() {
        let limiter = SourceLimiter::new(2);
        let first = "192.0.2.1".parse::<IpAddr>().expect("地址应有效");
        let second = "192.0.2.2".parse::<IpAddr>().expect("地址应有效");
        let first_permit = limiter.try_acquire(first).expect("首个额度应可用");
        let _second_permit = limiter.try_acquire(first).expect("第二个额度应可用");
        assert!(limiter.try_acquire(first).is_none());
        assert!(limiter.try_acquire(second).is_some());
        drop(first_permit);
        assert!(limiter.try_acquire(first).is_some());
    }

    #[test]
    fn ipv4_mapped_source_cannot_bypass_limit() {
        let limiter = SourceLimiter::new(1);
        let ipv4 = "192.0.2.1".parse::<IpAddr>().expect("地址应有效");
        let mapped = "::ffff:192.0.2.1"
            .parse::<IpAddr>()
            .expect("映射地址应有效");
        let _permit = limiter.try_acquire(ipv4).expect("额度应可用");
        assert!(limiter.try_acquire(mapped).is_none());
    }
}
