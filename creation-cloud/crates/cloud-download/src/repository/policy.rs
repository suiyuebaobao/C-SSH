//! 以事务锁读写更新策略草稿、追加式发布记录和正式资产集合。

use cloud_domain::{AppError, AppResult};
use cloud_store::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::model::{
    ForcedIdentityRow, PolicyAssetRow, PolicyTargetRow, PublishedUpdatePolicyRow,
    UpdatePolicyDraftRow,
};

pub(crate) struct PolicyAudit<'a> {
    pub action: &'a str,
    pub revision: i64,
    pub enabled: bool,
    pub forced_count: usize,
    pub target_release_id: Option<Uuid>,
    pub sha256_enabled: bool,
}

pub(crate) async fn draft(pool: &PgPool) -> AppResult<UpdatePolicyDraftRow> {
    sqlx::query_as(
        "SELECT revision, enabled, forced_versions, target_release_id, sha256_enabled, updated_at \
         FROM update_policy_draft WHERE singleton = TRUE",
    )
    .fetch_one(pool)
    .await
    .map_err(read_error)
}

pub(crate) async fn lock_draft(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<UpdatePolicyDraftRow> {
    sqlx::query_as(
        "SELECT revision, enabled, forced_versions, target_release_id, sha256_enabled, updated_at \
         FROM update_policy_draft WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)
}

pub(crate) async fn current(pool: &PgPool) -> AppResult<Option<PublishedUpdatePolicyRow>> {
    sqlx::query_as(
        "SELECT publication.revision, publication.enabled, publication.forced_versions, \
                publication.target_release_id, release.version AS target_version, \
                publication.sha256_enabled, publication.published_at, publication.published_by \
         FROM update_policy_publication_state AS state \
         JOIN update_policy_publications AS publication \
           ON publication.revision = state.current_revision \
         LEFT JOIN releases AS release ON release.id = publication.target_release_id \
         WHERE state.singleton = TRUE",
    )
    .fetch_optional(pool)
    .await
    .map_err(read_error)
}

pub(crate) async fn targets(pool: &PgPool) -> AppResult<Vec<PolicyTargetRow>> {
    sqlx::query_as(
        "SELECT release.id, release.version, release.published_at, \
                COUNT(DISTINCT asset.id) AS asset_count, \
                COUNT(DISTINCT asset.id) FILTER (WHERE \
                    (asset.platform = 'windows' AND asset.architecture = 'x86_64' \
                     AND asset.package_kind IN ('exe', 'msi', 'zip')) \
                    OR (asset.platform = 'android' AND asset.architecture = 'aarch64' \
                        AND asset.package_kind = 'apk')) AS formal_asset_count, \
                COUNT(DISTINCT asset.id) FILTER (WHERE asset.platform = 'windows' \
                    AND asset.package_kind IN ('exe', 'msi', 'zip') \
                    AND asset.updater_signature IS NOT NULL) AS required_signature_count, \
                COUNT(DISTINCT asset.id) FILTER (WHERE source.source_kind = 'local' \
                    AND source.enabled = TRUE) AS local_source_count \
         FROM releases AS release \
         LEFT JOIN release_assets AS asset ON asset.release_id = release.id \
         LEFT JOIN release_sources AS source ON source.asset_id = asset.id \
         WHERE release.status = 'published' AND release.channel = 'stable' \
         GROUP BY release.id, release.version, release.published_at \
         ORDER BY release.published_at DESC, release.id DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(read_error)
}

pub(crate) async fn save_draft(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    expected_revision: i64,
    enabled: bool,
    forced_versions: &[String],
    target_release_id: Option<Uuid>,
    sha256_enabled: bool,
) -> AppResult<UpdatePolicyDraftRow> {
    sqlx::query_as(
        "UPDATE update_policy_draft SET revision = revision + 1, enabled = $3, \
            forced_versions = $4, target_release_id = $5, sha256_enabled = $6, \
            updated_by = $2, updated_at = now() \
         WHERE singleton = TRUE AND revision = $1 \
         RETURNING revision, enabled, forced_versions, target_release_id, \
                   sha256_enabled, updated_at",
    )
    .bind(expected_revision)
    .bind(actor_id)
    .bind(enabled)
    .bind(forced_versions)
    .bind(target_release_id)
    .bind(sha256_enabled)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::Conflict("版本策略草稿已变化，请刷新后重试".into()))
}

pub(crate) async fn lock_publication_revision(
    transaction: &mut Transaction<'_, Postgres>,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar(
        "SELECT current_revision FROM update_policy_publication_state \
         WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)
}

pub(crate) async fn policy_assets(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> AppResult<Vec<PolicyAssetRow>> {
    sqlx::query_as(
        "SELECT asset.id, asset.platform, asset.architecture, \
                asset.package_kind, asset.file_name, asset.byte_size, asset.sha256, \
                asset.updater_signature, source.id AS source_id, source.local_path \
         FROM releases AS release \
         JOIN release_assets AS asset ON asset.release_id = release.id \
         JOIN release_sources AS source ON source.asset_id = asset.id \
         WHERE release.id = $1 AND release.status = 'published' AND release.channel = 'stable' \
           AND source.enabled = TRUE AND source.source_kind = 'local' \
         ORDER BY asset.id, source.sort_order, source.created_at, source.id \
         FOR SHARE OF release, asset, source",
    )
    .bind(release_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(write_error)
}

pub(crate) async fn target_version(
    transaction: &mut Transaction<'_, Postgres>,
    release_id: Uuid,
) -> AppResult<String> {
    sqlx::query_scalar(
        "SELECT version FROM releases WHERE id = $1 AND status = 'published' \
         AND channel = 'stable' FOR SHARE",
    )
    .bind(release_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(write_error)?
    .ok_or_else(|| AppError::Conflict("目标正式版本不存在或尚未发布".into()))
}

pub(crate) async fn forced_identities(
    transaction: &mut Transaction<'_, Postgres>,
    versions: &[String],
) -> AppResult<Vec<ForcedIdentityRow>> {
    sqlx::query_as(
        "SELECT release.version, asset.platform, asset.architecture, asset.package_kind, \
                asset.sha256 AS asset_sha256, asset.installed_sha256 \
         FROM releases AS release \
         JOIN release_assets AS asset ON asset.release_id = release.id \
         WHERE release.status = 'published' AND release.channel = 'stable' \
           AND release.version = ANY($1) AND ( \
             (asset.platform = 'windows' AND asset.architecture = 'x86_64' \
              AND asset.package_kind IN ('exe', 'msi', 'zip')) \
             OR (asset.platform = 'android' AND asset.architecture = 'aarch64' \
                 AND asset.package_kind = 'apk')) \
         ORDER BY release.version, asset.platform, asset.architecture, asset.package_kind \
         FOR SHARE OF release, asset",
    )
    .bind(versions)
    .fetch_all(&mut **transaction)
    .await
    .map_err(write_error)
}

pub(crate) async fn installed_sha256(
    pool: &PgPool,
    version: &str,
    platform: &str,
    architecture: &str,
    package_kind: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT asset.installed_sha256 \
         FROM releases AS release \
         JOIN release_assets AS asset ON asset.release_id = release.id \
         WHERE release.status = 'published' AND release.channel = 'stable' \
           AND release.version = $1 AND asset.platform = $2 \
           AND asset.architecture = $3 AND asset.package_kind = $4",
    )
    .bind(version)
    .bind(platform)
    .bind(architecture)
    .bind(package_kind)
    .fetch_optional(pool)
    .await
    .map_err(read_error)?
    .ok_or_else(|| AppError::Storage("强制版本缺少当前安装形态身份".into()))
}

pub(crate) async fn publish(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    revision: i64,
    draft: &UpdatePolicyDraftRow,
) -> AppResult<PublishedUpdatePolicyRow> {
    let target_version = if let Some(id) = draft.target_release_id {
        Some(target_version(transaction, id).await?)
    } else {
        None
    };
    let row = sqlx::query_as(
        "INSERT INTO update_policy_publications \
         (revision, enabled, forced_versions, target_release_id, sha256_enabled, published_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING revision, enabled, forced_versions, target_release_id, \
                   $7::text AS target_version, sha256_enabled, published_at, published_by",
    )
    .bind(revision)
    .bind(draft.enabled)
    .bind(&draft.forced_versions)
    .bind(draft.target_release_id)
    .bind(draft.sha256_enabled)
    .bind(actor_id)
    .bind(target_version)
    .fetch_one(&mut **transaction)
    .await
    .map_err(write_error)?;
    sqlx::query(
        "UPDATE update_policy_publication_state SET current_revision = $1, updated_at = now() \
         WHERE singleton = TRUE",
    )
    .bind(revision)
    .execute(&mut **transaction)
    .await
    .map_err(write_error)?;
    Ok(row)
}

pub(crate) async fn audit(
    transaction: &mut Transaction<'_, Postgres>,
    actor_id: Uuid,
    audit: PolicyAudit<'_>,
) -> AppResult<()> {
    let request_id =
        cloud_domain::current_request_id().unwrap_or_else(|| Uuid::now_v7().to_string());
    let details = serde_json::json!({
        "revision": audit.revision,
        "enabled": audit.enabled,
        "forced_version_count": audit.forced_count,
        "target_release_id": audit.target_release_id,
        "sha256_enabled": audit.sha256_enabled
    });
    sqlx::query(
        "INSERT INTO audit_events (id, actor_account_id, action, resource_kind, \
         resource_id, outcome, request_id, details) \
         VALUES ($1, $2, $3, 'update_policy', 'singleton', 'success', $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(actor_id)
    .bind(audit.action)
    .bind(request_id)
    .bind(details)
    .execute(&mut **transaction)
    .await
    .map_err(|_| AppError::Storage("保存版本策略审计失败".into()))?;
    Ok(())
}

fn read_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("读取版本策略失败".into())
}

fn write_error(_error: sqlx::Error) -> AppError {
    AppError::Storage("保存版本策略失败".into())
}
