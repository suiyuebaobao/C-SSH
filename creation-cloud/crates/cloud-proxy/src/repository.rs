//! 封装官方代理目录、短凭据事务和内网鉴权所需的 PostgreSQL 查询。

use chrono::{DateTime, Utc};
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{ProxyError, error};

#[path = "repository_tunnel.rs"]
mod tunnel;
pub(crate) use tunnel::{admission_candidate, admit_tunnel, close_tunnel, continuation_lease};

const ISSUE_LIMIT_PER_MINUTE: i64 = 30;
const ACTIVE_ADMISSION_LIMIT: i64 = 64;
const CLEANUP_BATCH_SIZE: i64 = 256;

#[derive(sqlx::FromRow)]
pub(crate) struct NodeEndpointRecord {
    pub node_id: Uuid,
    pub display_name: String,
    pub region: String,
    pub protocol: String,
    pub host: String,
    pub port: i32,
    pub tls_server_name: String,
}

#[derive(sqlx::FromRow)]
pub(crate) struct IssuedCredentialRecord {
    pub id: Uuid,
    pub node_id: Uuid,
    pub username: String,
    pub key_version: i32,
    pub generation: i64,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub(crate) struct NewCredential<'a> {
    pub id: Uuid,
    pub mutation_id: Uuid,
    pub node_id: Uuid,
    pub username: &'a str,
    pub key_version: i32,
    pub password_digest: &'a [u8; 32],
    pub target_digest: &'a [u8; 32],
    pub request_digest: &'a [u8; 32],
    pub ttl_seconds: i64,
}

pub(crate) async fn list_nodes(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
    environment: &str,
) -> Result<Vec<NodeEndpointRecord>, ProxyError> {
    validate_identity(pool, session, device_id).await?;
    sqlx::query_as::<_, NodeEndpointRecord>(
        r#"
        SELECT node.id AS node_id, node.display_name, node.region,
               endpoint.protocol, endpoint.host, endpoint.port,
               endpoint.tls_server_name
        FROM proxy_nodes AS node
        JOIN proxy_node_endpoints AS endpoint ON endpoint.node_id = node.id
        WHERE node.environment = $1
          AND node.status = 'active'
          AND node.fail_closed_gateway_verified_at IS NOT NULL
          AND endpoint.enabled
          AND endpoint.protocol = 'http_connect'
          AND endpoint.transport_security = 'tls'
        ORDER BY node.display_name, node.id, endpoint.priority, endpoint.id
        "#,
    )
    .bind(environment)
    .fetch_all(pool)
    .await
    .map_err(error::storage)
}

pub(crate) async fn issue(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
    environment: &str,
    credential: NewCredential<'_>,
) -> Result<IssuedCredentialRecord, ProxyError> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    let account_credential_version = lock_identity(&mut transaction, session, device_id).await?;
    if let Some(existing) = find_mutation(
        &mut transaction,
        session.account_id,
        device_id,
        credential.mutation_id,
    )
    .await?
    {
        if !bool::from(existing.request_digest.ct_eq(credential.request_digest)) {
            return Err(ProxyError::MutationConflict);
        }
        transaction.commit().await.map_err(error::storage)?;
        return Ok(existing.credential);
    }
    tunnel::cleanup_expired_leases(&mut transaction, CLEANUP_BATCH_SIZE).await?;
    cleanup_expired_credentials(
        &mut transaction,
        session.account_id,
        device_id,
        CLEANUP_BATCH_SIZE,
    )
    .await?;
    enforce_issue_limits(&mut transaction, session.account_id, device_id).await?;
    validate_node(&mut transaction, credential.node_id, environment).await?;
    let generation = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(MAX(generation), 0) + 1
        FROM proxy_device_credentials
        WHERE account_id = $1 AND device_id = $2 AND node_id = $3
          AND target_digest = $4
        "#,
    )
    .bind(session.account_id)
    .bind(device_id)
    .bind(credential.node_id)
    .bind(credential.target_digest.as_slice())
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::storage)?;
    let inserted = sqlx::query_as::<_, IssuedCredentialRecord>(
        r#"
        INSERT INTO proxy_device_credentials (
            id, account_id, device_id, session_id, node_id, username,
            key_version, account_credential_version, password_digest,
            target_digest, mutation_id, request_digest, generation,
            issued_at, not_before, expires_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6,
            $7, $8, $9, $10, $11, $12, $13,
            now(), now(), now() + make_interval(secs => $14::double precision)
        )
        RETURNING id, node_id, username, key_version, generation, issued_at, expires_at
        "#,
    )
    .bind(credential.id)
    .bind(session.account_id)
    .bind(device_id)
    .bind(session.session_id)
    .bind(credential.node_id)
    .bind(credential.username)
    .bind(credential.key_version)
    .bind(account_credential_version)
    .bind(credential.password_digest.as_slice())
    .bind(credential.target_digest.as_slice())
    .bind(credential.mutation_id)
    .bind(credential.request_digest.as_slice())
    .bind(generation)
    .bind(credential.ttl_seconds as f64)
    .fetch_one(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(inserted)
}

async fn cleanup_expired_credentials(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    device_id: Uuid,
    batch_size: i64,
) -> Result<(), ProxyError> {
    sqlx::query(
        r#"
        DELETE FROM proxy_device_credentials AS credential
        WHERE credential.id IN (
            SELECT candidate.id
            FROM proxy_device_credentials AS candidate
            WHERE candidate.account_id = $1 AND candidate.device_id = $2
              AND candidate.expires_at <= now()
              AND NOT EXISTS (
                  SELECT 1 FROM proxy_tunnel_leases AS lease
                  WHERE lease.credential_id = candidate.id
                    AND lease.lease_expires_at > now()
              )
            ORDER BY candidate.expires_at, candidate.id
            LIMIT $3
            FOR UPDATE OF candidate SKIP LOCKED
        )
        "#,
    )
    .bind(account_id)
    .bind(device_id)
    .bind(batch_size)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}

async fn enforce_issue_limits(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    device_id: Uuid,
) -> Result<(), ProxyError> {
    let (recent, active) = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT
            count(*) FILTER (WHERE issued_at > now() - interval '1 minute')::BIGINT,
            count(*) FILTER (
                WHERE revoked_at IS NULL AND not_before <= now() AND expires_at > now()
            )::BIGINT
        FROM proxy_device_credentials
        WHERE account_id = $1 AND device_id = $2
        "#,
    )
    .bind(account_id)
    .bind(device_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(error::storage)?;
    if recent >= ISSUE_LIMIT_PER_MINUTE || active >= ACTIVE_ADMISSION_LIMIT {
        return Err(ProxyError::RateLimited);
    }
    Ok(())
}

pub(crate) async fn revoke(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
    credential_id: Uuid,
) -> Result<(), ProxyError> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    lock_identity(&mut transaction, session, device_id).await?;
    let revoked = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE proxy_device_credentials
        SET revoked_at = COALESCE(revoked_at, now())
        WHERE id = $1 AND account_id = $2 AND device_id = $3
        RETURNING id
        "#,
    )
    .bind(credential_id)
    .bind(session.account_id)
    .bind(device_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    if revoked.is_none() {
        return Err(ProxyError::CredentialNotFound);
    }
    transaction.commit().await.map_err(error::storage)?;
    Ok(())
}

async fn validate_identity(
    pool: &PgPool,
    session: &AuthenticatedSession,
    device_id: Uuid,
) -> Result<(), ProxyError> {
    let exists = sqlx::query_scalar::<_, bool>(identity_exists_sql())
        .bind(session.account_id)
        .bind(session.session_id)
        .bind(device_id)
        .fetch_one(pool)
        .await
        .map_err(error::storage)?;
    if !exists {
        return Err(ProxyError::SessionChanged);
    }
    Ok(())
}

async fn lock_identity(
    transaction: &mut Transaction<'_, Postgres>,
    session: &AuthenticatedSession,
    device_id: Uuid,
) -> Result<i64, ProxyError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT account.credential_version
        FROM accounts AS account
        JOIN sessions AS session ON session.account_id = account.id
        JOIN devices AS device
          ON device.account_id = account.id
         AND device.active_session_reference_id = session.device_id
        WHERE account.id = $1 AND account.status = 'active'
          AND (account.role = 'admin' OR account.email_verified_at IS NOT NULL)
          AND session.id = $2 AND session.device_id = $3
          AND session.credential_version = account.credential_version
          AND session.revoked_at IS NULL
          AND session.expires_at > now()
          AND session.absolute_expires_at > now()
        FOR UPDATE OF account, session, device
        "#,
    )
    .bind(session.account_id)
    .bind(session.session_id)
    .bind(device_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?
    .ok_or(ProxyError::SessionChanged)
}

fn identity_exists_sql() -> &'static str {
    r#"
    SELECT EXISTS (
        SELECT 1
        FROM accounts AS account
        JOIN sessions AS session ON session.account_id = account.id
        JOIN devices AS device
          ON device.account_id = account.id
         AND device.active_session_reference_id = session.device_id
        WHERE account.id = $1 AND account.status = 'active'
          AND (account.role = 'admin' OR account.email_verified_at IS NOT NULL)
          AND session.id = $2 AND session.device_id = $3
          AND session.credential_version = account.credential_version
          AND session.revoked_at IS NULL
          AND session.expires_at > now()
          AND session.absolute_expires_at > now()
    )
    "#
}

async fn validate_node(
    transaction: &mut Transaction<'_, Postgres>,
    node_id: Uuid,
    environment: &str,
) -> Result<(), ProxyError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM proxy_nodes AS node
            WHERE node.id = $1 AND node.environment = $2
              AND node.status = 'active'
              AND node.fail_closed_gateway_verified_at IS NOT NULL
              AND EXISTS (
                  SELECT 1 FROM proxy_node_endpoints AS endpoint
                  WHERE endpoint.node_id = node.id
                    AND endpoint.enabled
                    AND endpoint.protocol = 'http_connect'
                    AND endpoint.transport_security = 'tls'
              )
        )
        "#,
    )
    .bind(node_id)
    .bind(environment)
    .fetch_one(&mut **transaction)
    .await
    .map_err(error::storage)?;
    if !exists {
        return Err(ProxyError::Unavailable);
    }
    Ok(())
}

struct MutationRecord {
    request_digest: Vec<u8>,
    credential: IssuedCredentialRecord,
}

async fn find_mutation(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    device_id: Uuid,
    mutation_id: Uuid,
) -> Result<Option<MutationRecord>, ProxyError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        node_id: Uuid,
        username: String,
        key_version: i32,
        generation: i64,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        request_digest: Vec<u8>,
    }
    let row = sqlx::query_as::<_, Row>(
        r#"
        SELECT id, node_id, username, key_version, generation,
               issued_at, expires_at, request_digest
        FROM proxy_device_credentials
        WHERE account_id = $1 AND device_id = $2 AND mutation_id = $3
        FOR UPDATE
        "#,
    )
    .bind(account_id)
    .bind(device_id)
    .bind(mutation_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(row.map(|row| MutationRecord {
        request_digest: row.request_digest,
        credential: IssuedCredentialRecord {
            id: row.id,
            node_id: row.node_id,
            username: row.username,
            key_version: row.key_version,
            generation: row.generation,
            issued_at: row.issued_at,
            expires_at: row.expires_at,
        },
    }))
}
