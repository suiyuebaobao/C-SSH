//! 身份或凭据变化时清除目标账号尚未结算的认证挑战。

use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::repository::map_write_error;

pub(crate) const DELETE_ACCOUNT_CHALLENGES_SQL: &str = r#"
    WITH email_deleted AS (
        DELETE FROM email_verification_challenges WHERE account_id = $1
    ), login_deleted AS (
        DELETE FROM login_verification_challenges WHERE account_id = $1
    )
    DELETE FROM password_reset_challenges WHERE account_id = $1
"#;

pub(crate) async fn delete_for_account(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> AppResult<()> {
    sqlx::query(DELETE_ACCOUNT_CHALLENGES_SQL)
        .bind(account_id)
        .execute(&mut **transaction)
        .await
        .map_err(map_write_error)?;
    Ok(())
}
