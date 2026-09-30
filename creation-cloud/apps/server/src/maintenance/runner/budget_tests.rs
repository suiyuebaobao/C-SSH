//! 验证业务执行预算会为终态收敛保留独立且有界的时间。

use super::*;

#[test]
fn execution_budget_reserves_a_bounded_terminal_phase() {
    let before = Instant::now();
    let budget = RunBudget::new(Duration::from_secs(10));
    assert!(budget.execution_deadline >= before + Duration::from_secs(10));
    assert_eq!(
        budget.terminal_deadline - budget.execution_deadline,
        TERMINAL_SETTLE_TIMEOUT
    );
}
