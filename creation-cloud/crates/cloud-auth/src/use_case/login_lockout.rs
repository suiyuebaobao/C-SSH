//! 计算账号级连续登录失败的持久锁定状态。

use chrono::{DateTime, Duration, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;

use crate::{repository, repository::login::LoginAccount, validation::LoginIdentifier};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FailureState {
    pub consecutive_failures: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

pub(crate) fn next_failure_state(
    current_failures: i32,
    previous_locked_until: Option<DateTime<Utc>>,
    failure_threshold: i32,
    lockout_minutes: i32,
    now: DateTime<Utc>,
) -> FailureState {
    let threshold = failure_threshold.max(1);
    let prior_cycle_expired = previous_locked_until.is_some_and(|until| until <= now);
    let base = if prior_cycle_expired {
        0
    } else {
        current_failures.max(0).min(threshold)
    };
    let consecutive_failures = base.saturating_add(1).min(threshold);
    let locked_until = (consecutive_failures >= threshold)
        .then(|| now + Duration::minutes(i64::from(lockout_minutes.max(1))));
    FailureState {
        consecutive_failures,
        locked_until,
    }
}

pub(crate) fn active_lock_error(
    locked_until: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<AppError> {
    let locked_until = locked_until.filter(|until| *until > now)?;
    let remaining_millis = (locked_until - now).num_milliseconds().max(1);
    let retry_after_seconds = u64::try_from(remaining_millis)
        .unwrap_or(u64::MAX)
        .saturating_add(999)
        / 1_000;
    Some(AppError::RateLimitedAfter {
        message: "登录失败次数过多，请稍后重试".to_owned(),
        retry_after_seconds: retry_after_seconds.max(1),
    })
}

pub(crate) fn ensure_not_locked(account: &LoginAccount, now: DateTime<Utc>) -> AppResult<()> {
    match active_lock_error(account.login_locked_until, now) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub(crate) async fn record_failed_password(
    pool: &PgPool,
    initial: &LoginAccount,
    identifier: &LoginIdentifier,
) -> AppResult<Option<AppError>> {
    let mut transaction = pool.begin().await.map_err(repository::error::storage)?;
    let Some(current) = repository::login::lock_by_id(&mut transaction, initial.id).await? else {
        let _ = transaction.rollback().await;
        return Ok(None);
    };
    if !super::login::snapshot_allows_session(initial, &current, identifier)
        || !can_accumulate_failures(&current)
    {
        let _ = transaction.rollback().await;
        return Ok(None);
    }
    let settings = repository::settings::lock(&mut transaction).await?;
    let now = Utc::now();
    if let Some(error) = active_lock_error(current.login_locked_until, now) {
        let _ = transaction.rollback().await;
        return Ok(Some(error));
    }
    let state = next_failure_state(
        current.consecutive_login_failures,
        current.login_locked_until,
        settings.login_failure_threshold,
        settings.login_lockout_minutes,
        now,
    );
    repository::login::update_login_failures(
        &mut transaction,
        current.id,
        state.consecutive_failures,
        state.locked_until,
    )
    .await?;
    transaction
        .commit()
        .await
        .map_err(repository::error::storage)?;
    Ok(active_lock_error(state.locked_until, now))
}

fn can_accumulate_failures(account: &LoginAccount) -> bool {
    account.status == "active"
        && (account.role == "admin"
            || (account.role == "user" && account.email_verified_at.is_some()))
}
