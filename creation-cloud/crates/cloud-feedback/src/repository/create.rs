//! 在账号级事务锁内校验 active 状态和双时间窗限额，再原子插入反馈。

use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{CreateFeedbackInput, FeedbackSubmission, model::FeedbackRow};

use super::error;

pub(crate) const ADVISORY_LOCK_SQL: &str =
    "SELECT pg_advisory_xact_lock(hashtextextended($1::uuid::text, 0))";
pub(crate) const ACTIVE_ACCOUNT_SQL: &str = "SELECT status FROM accounts WHERE id = $1 FOR SHARE";
pub(crate) const RATE_LIMIT_SQL: &str = "SELECT COUNT(*) FILTER (WHERE created_at >= now() - INTERVAL '10 minutes') AS recent, \
            COUNT(*) AS daily \
     FROM feedback_submissions \
     WHERE account_id = $1 AND created_at >= now() - INTERVAL '24 hours'";
pub(crate) const INSERT_SQL: &str = r#"
    INSERT INTO feedback_submissions (
        id, account_id, request_id, category, platform,
        app_version, title, description
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    RETURNING id, account_id, request_id, category, platform, app_version,
              title, description, status, version, redacted_at, redaction_reason,
              created_at, updated_at
"#;

pub(crate) async fn execute(
    pool: &PgPool,
    id: Uuid,
    account_id: Uuid,
    request_id: &str,
    input: &CreateFeedbackInput,
) -> AppResult<FeedbackSubmission> {
    let mut transaction = pool.begin().await.map_err(error::transaction)?;

    // PostgreSQL 事务级账号锁让并发请求串行完成计数与写入，避免检查后插入竞态。
    sqlx::query(ADVISORY_LOCK_SQL)
        .bind(account_id)
        .execute(&mut *transaction)
        .await
        .map_err(error::transaction)?;

    // 共享锁把账号 active 判定保持到提交，防止并发禁用与反馈写入相互穿越。
    let account_status = sqlx::query_scalar::<_, String>(ACTIVE_ACCOUNT_SQL)
        .bind(account_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(error::read)?;
    if account_status.as_deref() != Some("active") {
        return Err(AppError::Unauthorized("反馈账号不可用".to_owned()));
    }

    let (recent, daily) = sqlx::query_as::<_, (i64, i64)>(RATE_LIMIT_SQL)
        .bind(account_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(error::read)?;
    if rate_limited(recent, daily) {
        return Err(AppError::RateLimited(
            "反馈提交过于频繁，请稍后再试".to_owned(),
        ));
    }

    let row = sqlx::query_as::<_, FeedbackRow>(INSERT_SQL)
        .bind(id)
        .bind(account_id)
        .bind(request_id)
        .bind(input.category.as_str())
        .bind(input.platform.as_str())
        .bind(input.app_version.as_deref())
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&mut *transaction)
        .await
        .map_err(error::write)?;

    transaction.commit().await.map_err(error::transaction)?;
    FeedbackSubmission::try_from(row)
}

pub(crate) const fn rate_limited(recent: i64, daily: i64) -> bool {
    recent >= 3 || daily >= 20
}
