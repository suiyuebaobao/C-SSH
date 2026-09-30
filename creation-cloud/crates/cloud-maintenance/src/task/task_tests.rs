//! 验证固定维护任务名称往返与 advisory lock 身份互不冲突。

use super::*;

#[test]
fn names_round_trip_and_lock_identities_are_independent() {
    let mut identities = std::collections::BTreeSet::new();
    for task in MaintenanceTask::ALL {
        assert_eq!(task.as_str().parse(), Ok(task));
        assert!(identities.insert(task.advisory_lock_identity()));
    }
}
