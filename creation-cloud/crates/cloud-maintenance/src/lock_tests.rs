//! 验证维护专用连接的会话上界和显式释放结果分类。

use super::*;

#[test]
fn session_sql_bound_stays_inside_global_shutdown_window() {
    assert_eq!(LOCK_WAIT_TIMEOUT_SECONDS, 3);
    assert_eq!(SESSION_STATEMENT_TIMEOUT_SECONDS, 25);
    assert_eq!(SESSION_IDLE_TRANSACTION_TIMEOUT_SECONDS, 25);
}

#[test]
fn successful_unlock_is_not_reclassified_when_consuming_close_fails() {
    assert!(finish_release(true, false).is_ok());
    assert!(finish_release(false, true).is_err());
}
