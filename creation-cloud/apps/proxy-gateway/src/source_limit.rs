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
