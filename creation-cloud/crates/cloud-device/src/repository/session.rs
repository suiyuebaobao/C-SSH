use chrono::{DateTime, Utc};
use cloud_domain::{AppResult, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::SessionView;

use super::error;

type SessionRow = (
    Uuid,
    String,
    Uuid,
    Option<String>,
    Option<String>,
    Option<Uuid>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    DateTime<Utc>,
    DateTime<Utc>,
    DateTime<Utc>,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

const SESSION_PROJECTION_SQL: &str = "SELECT session.id, \
        CASE WHEN session.revoked_at IS NULL \
                  AND account.status = 'active' \
                  AND session.credential_version = account.credential_version \
                  AND session.expires_at > now() \
                  AND session.absolute_expires_at > now() \
                  AND session.last_seen_at >= now() - interval '5 minutes' \
                  AND (session.device_id IS NULL \
                       OR (device.id IS NOT NULL AND device.revoked_at IS NULL)) \
             THEN 'online' ELSE 'offline' END, \
        session.account_id, account.email, account.admin_login_name, \
        session.device_id, device.name, host(session.last_login_ip), \
        session.user_agent, session.client_version, session.device_fingerprint, \
        session.created_at, session.last_seen_at, session.expires_at, \
        session.absolute_expires_at, session.revoked_at \
     FROM sessions AS session \
     JOIN accounts AS account ON account.id = session.account_id \
     LEFT JOIN devices AS device ON device.account_id = session.account_id \
        AND device.id = session.device_id";

pub(crate) async fn list_for_account(
    pool: &PgPool,
    account_id: Uuid,
    current_session_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<SessionView>, i64)> {
    let sql = format!(
        "{SESSION_PROJECTION_SQL} WHERE session.account_id = $1 \
         ORDER BY session.last_seen_at DESC, session.id LIMIT $2 OFFSET $3"
    );
    let rows = sqlx::query_as::<_, SessionRow>(&sql)
        .bind(account_id)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(error::storage)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE account_id = $1")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map_err(error::storage)?;
    Ok((
        rows.into_iter()
            .map(|row| view(row, current_session_id))
            .collect(),
        total,
    ))
}

pub(crate) async fn list_all(
    pool: &PgPool,
    current_session_id: Uuid,
    page: PageQuery,
) -> AppResult<(Vec<SessionView>, i64)> {
    let sql = format!(
        "{SESSION_PROJECTION_SQL} \
         ORDER BY session.last_seen_at DESC, session.id LIMIT $1 OFFSET $2"
    );
    let rows = sqlx::query_as::<_, SessionRow>(&sql)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(error::storage)?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .map_err(error::storage)?;
    Ok((
        rows.into_iter()
            .map(|row| view(row, current_session_id))
            .collect(),
        total,
    ))
}

pub(crate) async fn revoke_for_account(
    pool: &PgPool,
    actor_account_id: Uuid,
    current_session_id: Uuid,
    session_id: Uuid,
) -> AppResult<bool> {
    revoke(
        pool,
        actor_account_id,
        current_session_id,
        session_id,
        actor_account_id,
    )
    .await
}

pub(crate) async fn delete_any(
    pool: &PgPool,
    actor_account_id: Uuid,
    session_id: Uuid,
) -> AppResult<bool> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let target_account_id =
        sqlx::query_scalar::<_, Uuid>("DELETE FROM sessions WHERE id = $1 RETURNING account_id")
            .bind(session_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(error::storage)?;
    if target_account_id.is_none() {
        transaction.rollback().await.map_err(error::storage)?;
        return Ok(false);
    }
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind,
         resource_id, outcome, request_id, details)
         VALUES ($1, $2, 'session.admin_delete', 'session', $3, 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_account_id)
    .bind(session_id.to_string())
    .bind(request_id)
    .bind(serde_json::json!({}))
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    cloud_domain::mark_semantic_audit_recorded();
    Ok(true)
}

async fn revoke(
    pool: &PgPool,
    actor_account_id: Uuid,
    current_session_id: Uuid,
    session_id: Uuid,
    account_scope: Uuid,
) -> AppResult<bool> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let target_account_id = sqlx::query_scalar::<_, Uuid>(
        "UPDATE sessions SET revoked_at = now()
         WHERE id = $1 AND account_id = $2 AND revoked_at IS NULL
         RETURNING account_id",
    )
    .bind(session_id)
    .bind(account_scope)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let Some(target_account_id) = target_account_id else {
        transaction.rollback().await.map_err(error::storage)?;
        return Ok(false);
    };
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = serde_json::json!({
        "target_account_id": target_account_id,
        "was_current": session_id == current_session_id
    });
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, 'session.revoke', 'session', $3, 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_account_id)
    .bind(session_id.to_string())
    .bind(request_id)
    .bind(details)
    .execute(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    cloud_domain::mark_semantic_audit_recorded();
    Ok(true)
}

fn view(row: SessionRow, current_session_id: Uuid) -> SessionView {
    SessionView {
        session_id: row.0,
        status: row.1,
        is_current: row.0 == current_session_id,
        account_id: row.2,
        account_label: account_label(row.3.as_deref(), row.4.as_deref()),
        device_id: row.5,
        device_name: row.6,
        last_login_ip: row.7,
        user_agent: row.8,
        client_version: row.9,
        device_fingerprint: row.10,
        created_at: row.11,
        last_seen_at: row.12,
        idle_expires_at: row.13,
        absolute_expires_at: row.14,
        revoked_at: row.15,
    }
}

fn account_label(email: Option<&str>, admin_login_name: Option<&str>) -> String {
    email
        .map(mask_email)
        .or_else(|| admin_login_name.map(mask_label))
        .unwrap_or_else(|| "***".to_owned())
}

fn mask_email(value: &str) -> String {
    let Some((local, domain)) = value.split_once('@') else {
        return "***".to_owned();
    };
    let local = mask_label(local);
    let domain = match domain.rsplit_once('.') {
        Some((name, suffix)) if !name.is_empty() && !suffix.is_empty() => {
            format!("{}.{}", mask_label(name), suffix)
        }
        _ => mask_label(domain),
    };
    format!("{local}@{domain}")
}

fn mask_label(value: &str) -> String {
    value
        .chars()
        .next()
        .map_or_else(|| "***".to_owned(), |first| format!("{first}***"))
}

#[cfg(test)]
mod tests {
    use super::{SESSION_PROJECTION_SQL, account_label};

    #[test]
    fn public_status_has_only_online_and_offline_branches() {
        assert!(SESSION_PROJECTION_SQL.contains("THEN 'online' ELSE 'offline' END"));
        assert!(SESSION_PROJECTION_SQL.contains("interval '5 minutes'"));
        assert!(SESSION_PROJECTION_SQL.contains("credential_version = account.credential_version"));
        assert!(!SESSION_PROJECTION_SQL.contains("token_hash"));
        assert!(!SESSION_PROJECTION_SQL.contains("password_hash"));
    }

    #[test]
    fn account_label_is_masked() {
        assert_eq!(
            account_label(Some("person@example.com"), None),
            "p***@e***.com"
        );
        assert_eq!(account_label(None, Some("ops-admin")), "o***");
    }
}
