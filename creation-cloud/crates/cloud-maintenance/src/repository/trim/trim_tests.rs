//! 验证维护历史串行锁定与确定性保留一百条的 SQL 契约。

use super::*;

#[test]
fn trim_sql_locks_first_and_keeps_a_deterministic_hundred_rows() {
    assert!(LOCK_TASK_STATE_SQL.contains("FOR UPDATE"));
    assert!(
        DELETE_OLD_COMPLETED_HISTORY_SQL
            .contains("CASE WHEN run_id = $2 OR run_id = $3 THEN 0 ELSE 1 END")
    );
    assert!(DELETE_OLD_COMPLETED_HISTORY_SQL.contains("started_at DESC,\n            run_id DESC"));
    assert!(DELETE_OLD_COMPLETED_HISTORY_SQL.contains("OFFSET $4"));
    assert_eq!(COMPLETED_HISTORY_LIMIT, 100);
}
