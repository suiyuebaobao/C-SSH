//! 分别执行本人所有权单查和管理员显式详情单查。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::model::FeedbackRow;

use super::error;

pub(crate) const COLUMNS: &str = "id, account_id, request_id, category, platform, app_version, \
                       title, description, status, version, redacted_at, redaction_reason, \
                       created_at, updated_at";

pub(crate) async fn owned(
    pool: &PgPool,
    account_id: Uuid,
    id: Uuid,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(&format!(
        "SELECT {COLUMNS} FROM feedback_submissions WHERE id = $1 AND account_id = $2"
    ))
    .bind(id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(error::read)
}

pub(crate) async fn management(pool: &PgPool, id: Uuid) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(&format!(
        "SELECT {COLUMNS} FROM feedback_submissions WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(error::read)
}

pub(crate) async fn management_for_update(
    connection: &mut PgConnection,
    id: Uuid,
) -> AppResult<Option<FeedbackRow>> {
    sqlx::query_as::<_, FeedbackRow>(&format!(
        "SELECT {COLUMNS} FROM feedback_submissions WHERE id = $1 FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(connection)
    .await
    .map_err(error::read)
}
