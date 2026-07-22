//! 跨取消边界保存本轮已经提交的非敏感计数，让失败终态不把真实进度归零。

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use cloud_maintenance::TaskExecutionReport;

#[derive(Clone, Default)]
pub struct CommittedProgress {
    inner: Arc<ProgressCounters>,
}

#[derive(Default)]
struct ProgressCounters {
    examined: AtomicU64,
    changed: AtomicU64,
    healthy: AtomicU64,
    issues: AtomicU64,
}

impl CommittedProgress {
    pub fn add(&self, report: TaskExecutionReport) {
        saturating_add(&self.inner.examined, report.examined_count);
        saturating_add(&self.inner.changed, report.changed_count);
        saturating_add(&self.inner.healthy, report.healthy_count);
        saturating_add(&self.inner.issues, report.issue_count);
    }

    #[must_use]
    pub fn snapshot(&self) -> TaskExecutionReport {
        TaskExecutionReport {
            examined_count: self.inner.examined.load(Ordering::Relaxed),
            changed_count: self.inner.changed.load(Ordering::Relaxed),
            healthy_count: self.inner.healthy.load(Ordering::Relaxed),
            issue_count: self.inner.issues.load(Ordering::Relaxed),
            observation: None,
        }
    }
}

fn saturating_add(counter: &AtomicU64, value: u64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}

#[cfg(test)]
#[path = "progress_tests.rs"]
mod tests;
