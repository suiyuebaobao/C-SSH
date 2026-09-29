//! 官方代理 admission 与 opaque tunnel lease 的 PostgreSQL 原子边界。

use chrono::{DateTime, Utc};
use cloud_store::PgPool;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{ProxyError, error};

const CONTINUATION_LEASE_SECONDS: i64 = 10;
const CLEANUP_BATCH_SIZE: i64 = 256;

#[derive(sqlx::FromRow)]
pub(crate) struct AdmissionCandidate {
    pub credential_id: Uuid,
    pub node_id: Uuid,
    pub key_version: i32,
    pub password_digest: Vec<u8>,
    pub target_digest: Vec<u8>,
}

pub(crate) async fn admission_candidate(
    pool: &PgPool,
    node_token_digest: &[u8; 32],
    username: &str,
    environment: &str,
) -> Result<Option<AdmissionCandidate>, ProxyError> {
    sqlx::query_as::<_, AdmissionCandidate>(
        r#"
        SELECT credential.id AS credential_id, credential.node_id, credential.key_version,
               credential.password_digest, credential.target_digest
        FROM proxy_node_auth_keys AS auth_key
        JOIN proxy_nodes AS node ON node.id = auth_key.node_id
        JOIN proxy_device_credentials AS credential ON credential.node_id = node.id
        JOIN accounts AS account ON account.id = credential.account_id
        JOIN devices AS device
          ON device.account_id = credential.account_id
         AND device.active_session_reference_id = credential.device_id
        JOIN sessions AS session
          ON session.id = credential.session_id
         AND session.account_id = credential.account_id
         AND session.device_id = credential.device_id
        WHERE auth_key.token_digest = $1
          AND auth_key.revoked_at IS NULL
          AND auth_key.not_before <= now()
          AND (auth_key.expires_at IS NULL OR auth_key.expires_at > now())
          AND node.environment = $2
          AND node.status = 'active'
          AND node.fail_closed_gateway_verified_at IS NOT NULL
          AND EXISTS (
              SELECT 1 FROM proxy_node_endpoints AS endpoint
              WHERE endpoint.node_id = node.id
                AND endpoint.enabled
                AND endpoint.protocol = 'http_connect'
                AND endpoint.transport_security = 'tls'
          )
          AND credential.username = $3
          AND credential.revoked_at IS NULL
          AND credential.not_before <= now()
          AND credential.expires_at > now()
          AND account.status = 'active'
          AND (account.role = 'admin' OR account.email_verified_at IS NOT NULL)
          AND account.credential_version = credential.account_credential_version
          AND session.credential_version = account.credential_version
          AND session.revoked_at IS NULL
          AND session.expires_at > now()
          AND session.absolute_expires_at > now()
        LIMIT 1
        "#,
    )
    .bind(node_token_digest.as_slice())
    .bind(environment)
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(error::storage)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn admit_tunnel(
    pool: &PgPool,
    node_token_digest: &[u8; 32],
    environment: &str,
    connection_id: Uuid,
    candidate: &AdmissionCandidate,
    password_digest: &[u8; 32],
    target_digest: &[u8; 32],
) -> Result<Option<DateTime<Utc>>, ProxyError> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    cleanup_expired_leases(&mut transaction, CLEANUP_BATCH_SIZE).await?;
    let lease_expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        WITH eligible AS (
            SELECT credential.id AS credential_id, credential.account_id,
                   credential.device_id, credential.node_id,
                   LEAST(
                       now() + make_interval(secs => $8::double precision),
                       session.expires_at,
                       session.absolute_expires_at,
                       COALESCE(auth_key.expires_at, 'infinity'::timestamptz)
                   ) AS lease_expires_at
            FROM proxy_node_auth_keys AS auth_key
            JOIN proxy_nodes AS node ON node.id = auth_key.node_id
            JOIN proxy_device_credentials AS credential ON credential.node_id = node.id
            JOIN accounts AS account ON account.id = credential.account_id
            JOIN devices AS device
              ON device.account_id = credential.account_id
             AND device.active_session_reference_id = credential.device_id
            JOIN sessions AS session
              ON session.id = credential.session_id
             AND session.account_id = credential.account_id
             AND session.device_id = credential.device_id
            WHERE auth_key.token_digest = $1
              AND auth_key.revoked_at IS NULL
              AND auth_key.not_before <= now()
              AND (auth_key.expires_at IS NULL OR auth_key.expires_at > now())
              AND node.environment = $2
              AND node.status = 'active'
              AND node.fail_closed_gateway_verified_at IS NOT NULL
              AND EXISTS (
                  SELECT 1 FROM proxy_node_endpoints AS endpoint
                  WHERE endpoint.node_id = node.id
                    AND endpoint.enabled
                    AND endpoint.protocol = 'http_connect'
                    AND endpoint.transport_security = 'tls'
              )
              AND credential.id = $3
              AND credential.key_version = $4
              AND credential.password_digest = $5
              AND credential.target_digest = $6
              AND credential.revoked_at IS NULL
              AND credential.not_before <= now()
              AND credential.expires_at > now()
              AND account.status = 'active'
              AND (account.role = 'admin' OR account.email_verified_at IS NOT NULL)
              AND account.credential_version = credential.account_credential_version
              AND session.credential_version = account.credential_version
              AND session.revoked_at IS NULL
              AND session.expires_at > now()
              AND session.absolute_expires_at > now()
            FOR UPDATE OF auth_key, node, credential, account, device, session
        ), admitted AS (
            INSERT INTO proxy_tunnel_leases (
                connection_id, credential_id, account_id, device_id, node_id,
                admitted_at, last_authorized_at, lease_expires_at
            )
            SELECT $7, credential_id, account_id, device_id, node_id,
                   now(), now(), lease_expires_at
            FROM eligible
            ON CONFLICT (connection_id) DO UPDATE
            SET last_authorized_at = now(),
                lease_expires_at = EXCLUDED.lease_expires_at
            WHERE proxy_tunnel_leases.credential_id = EXCLUDED.credential_id
              AND proxy_tunnel_leases.account_id = EXCLUDED.account_id
              AND proxy_tunnel_leases.device_id = EXCLUDED.device_id
              AND proxy_tunnel_leases.node_id = EXCLUDED.node_id
            RETURNING lease_expires_at
        )
        SELECT lease_expires_at FROM admitted
        "#,
    )
    .bind(node_token_digest.as_slice())
    .bind(environment)
    .bind(candidate.credential_id)
    .bind(candidate.key_version)
    .bind(password_digest.as_slice())
    .bind(target_digest.as_slice())
    .bind(connection_id)
    .bind(CONTINUATION_LEASE_SECONDS as f64)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(lease_expires_at)
}

pub(crate) async fn continuation_lease(
    pool: &PgPool,
    node_token_digest: &[u8; 32],
    environment: &str,
    connection_id: Uuid,
) -> Result<Option<DateTime<Utc>>, ProxyError> {
    let mut transaction = pool.begin().await.map_err(error::storage)?;
    cleanup_expired_leases(&mut transaction, CLEANUP_BATCH_SIZE).await?;
    let lease_expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        WITH eligible AS (
            SELECT lease.connection_id,
                   LEAST(
                       now() + make_interval(secs => $4::double precision),
                       session.expires_at,
                       session.absolute_expires_at,
                       COALESCE(auth_key.expires_at, 'infinity'::timestamptz)
                   ) AS lease_expires_at
            FROM proxy_tunnel_leases AS lease
            JOIN proxy_device_credentials AS credential
              ON credential.id = lease.credential_id
             AND credential.account_id = lease.account_id
             AND credential.device_id = lease.device_id
             AND credential.node_id = lease.node_id
            JOIN proxy_nodes AS node ON node.id = lease.node_id
            JOIN proxy_node_auth_keys AS auth_key ON auth_key.node_id = node.id
            JOIN accounts AS account ON account.id = lease.account_id
            JOIN devices AS device
              ON device.account_id = lease.account_id
             AND device.active_session_reference_id = lease.device_id
            JOIN sessions AS session
              ON session.id = credential.session_id
             AND session.account_id = lease.account_id
             AND session.device_id = lease.device_id
            WHERE lease.connection_id = $1
              AND lease.lease_expires_at > now()
              AND auth_key.token_digest = $2
              AND auth_key.revoked_at IS NULL
              AND auth_key.not_before <= now()
              AND (auth_key.expires_at IS NULL OR auth_key.expires_at > now())
              AND node.environment = $3
              AND node.status = 'active'
              AND node.fail_closed_gateway_verified_at IS NOT NULL
              AND EXISTS (
                  SELECT 1 FROM proxy_node_endpoints AS endpoint
                  WHERE endpoint.node_id = node.id
                    AND endpoint.enabled
                    AND endpoint.protocol = 'http_connect'
                    AND endpoint.transport_security = 'tls'
              )
              AND credential.revoked_at IS NULL
              AND credential.not_before <= now()
              AND account.status = 'active'
              AND (account.role = 'admin' OR account.email_verified_at IS NOT NULL)
              AND account.credential_version = credential.account_credential_version
              AND session.credential_version = account.credential_version
              AND session.revoked_at IS NULL
              AND session.expires_at > now()
              AND session.absolute_expires_at > now()
            FOR UPDATE OF lease, auth_key, node, credential, account, device, session
        )
        UPDATE proxy_tunnel_leases AS lease
        SET last_authorized_at = now(), lease_expires_at = eligible.lease_expires_at
        FROM eligible
        WHERE lease.connection_id = eligible.connection_id
        RETURNING lease.lease_expires_at
        "#,
    )
    .bind(connection_id)
    .bind(node_token_digest.as_slice())
    .bind(environment)
    .bind(CONTINUATION_LEASE_SECONDS as f64)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(error::storage)?;
    transaction.commit().await.map_err(error::storage)?;
    Ok(lease_expires_at)
}

pub(crate) async fn close_tunnel(
    pool: &PgPool,
    node_token_digest: &[u8; 32],
    environment: &str,
    connection_id: Uuid,
) -> Result<(), ProxyError> {
    sqlx::query(
        r#"
        DELETE FROM proxy_tunnel_leases AS lease
        USING proxy_nodes AS node, proxy_node_auth_keys AS auth_key
        WHERE lease.connection_id = $1
          AND node.id = lease.node_id
          AND node.environment = $2
          AND auth_key.node_id = node.id
          AND auth_key.token_digest = $3
          AND auth_key.revoked_at IS NULL
          AND auth_key.not_before <= now()
          AND (auth_key.expires_at IS NULL OR auth_key.expires_at > now())
        "#,
    )
    .bind(connection_id)
    .bind(environment)
    .bind(node_token_digest.as_slice())
    .execute(pool)
    .await
    .map_err(error::storage)?;
    Ok(())
}

pub(super) async fn cleanup_expired_leases(
    transaction: &mut Transaction<'_, Postgres>,
    batch_size: i64,
) -> Result<(), ProxyError> {
    sqlx::query(
        r#"
        DELETE FROM proxy_tunnel_leases AS lease
        WHERE lease.connection_id IN (
            SELECT expired.connection_id
            FROM proxy_tunnel_leases AS expired
            WHERE expired.lease_expires_at <= now()
            ORDER BY expired.lease_expires_at, expired.connection_id
            LIMIT $1
            FOR UPDATE OF expired SKIP LOCKED
        )
        "#,
    )
    .bind(batch_size)
    .execute(&mut **transaction)
    .await
    .map_err(error::storage)?;
    Ok(())
}
