//! 账号级模型 API Key/Token 客户端密文存取。

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::ModelSecret;

use super::storage;

#[derive(sqlx::FromRow)]
struct SecretRow {
    revision: i64,
    ciphertext: Option<Vec<u8>>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

pub(crate) async fn get_secret(
    pool: &PgPool,
    account_id: Uuid,
    model_id: Uuid,
) -> AppResult<ModelSecret> {
    ensure_model(pool, model_id).await?;
    let row = sqlx::query_as::<_, SecretRow>(
        "SELECT revision, ciphertext, updated_at, deleted_at FROM account_model_secrets \
         WHERE account_id=$1 AND model_id=$2",
    )
    .bind(account_id)
    .bind(model_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法读取模型密文"))?;
    Ok(match row {
        Some(row) => secret_from_row(model_id, row),
        None => ModelSecret {
            model_id,
            revision: 0,
            present: false,
            ciphertext: None,
            updated_at: Utc::now(),
        },
    })
}

pub(crate) async fn put_secret(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
    model_id: Uuid,
    ciphertext: Vec<u8>,
    expected_revision: Option<i64>,
) -> AppResult<ModelSecret> {
    let mut tx = pool
        .begin()
        .await
        .map_err(storage("无法开始模型密文事务"))?;
    ensure_model_tx(&mut tx, model_id).await?;
    let current = sqlx::query_as::<_, SecretRow>(
        "SELECT revision, ciphertext, updated_at, deleted_at FROM account_model_secrets \
         WHERE account_id=$1 AND model_id=$2 FOR UPDATE",
    )
    .bind(account_id)
    .bind(model_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage("无法锁定模型密文"))?;
    let row = match current {
        None => {
            if expected_revision.is_some() {
                return Err(AppError::Conflict("模型密文尚不存在".to_owned()));
            }
            sqlx::query_as::<_, SecretRow>(
                "INSERT INTO account_model_secrets \
                 (account_id, model_id, ciphertext, source_device_id) VALUES ($1,$2,$3,$4) \
                 RETURNING revision, ciphertext, updated_at, deleted_at",
            )
            .bind(account_id)
            .bind(model_id)
            .bind(ciphertext)
            .bind(device_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(storage("无法创建模型密文"))?
        }
        Some(current) => {
            if expected_revision != Some(current.revision) {
                return Err(AppError::Conflict("模型密文 revision 已变化".to_owned()));
            }
            sqlx::query_as::<_, SecretRow>(
                "UPDATE account_model_secrets SET ciphertext=$3, source_device_id=$4, \
                 deleted_at=NULL, revision=revision+1, updated_at=now() \
                 WHERE account_id=$1 AND model_id=$2 \
                 RETURNING revision, ciphertext, updated_at, deleted_at",
            )
            .bind(account_id)
            .bind(model_id)
            .bind(ciphertext)
            .bind(device_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(storage("无法更新模型密文"))?
        }
    };
    tx.commit().await.map_err(storage("无法提交模型密文事务"))?;
    Ok(secret_from_row(model_id, row))
}

pub(crate) async fn delete_secret(
    pool: &PgPool,
    account_id: Uuid,
    device_id: Uuid,
    model_id: Uuid,
    expected_revision: i64,
) -> AppResult<ModelSecret> {
    let row = sqlx::query_as::<_, SecretRow>(
        "UPDATE account_model_secrets SET ciphertext=NULL, source_device_id=$4, \
         deleted_at=now(), revision=revision+1, updated_at=now() \
         WHERE account_id=$1 AND model_id=$2 AND revision=$3 AND deleted_at IS NULL \
         RETURNING revision, ciphertext, updated_at, deleted_at",
    )
    .bind(account_id)
    .bind(model_id)
    .bind(expected_revision)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(storage("无法删除模型密文"))?
    .ok_or_else(|| AppError::Conflict("模型密文不存在或 revision 已变化".to_owned()))?;
    Ok(secret_from_row(model_id, row))
}

fn secret_from_row(model_id: Uuid, row: SecretRow) -> ModelSecret {
    let present = row.deleted_at.is_none() && row.ciphertext.is_some();
    ModelSecret {
        model_id,
        revision: row.revision,
        present,
        ciphertext: row
            .ciphertext
            .filter(|_| present)
            .map(|value| STANDARD.encode(value)),
        updated_at: row.updated_at,
    }
}

async fn ensure_model(pool: &PgPool, model_id: Uuid) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM global_model_catalog \
         WHERE id=$1 AND deleted_at IS NULL)",
    )
    .bind(model_id)
    .fetch_one(pool)
    .await
    .map_err(storage("无法核验模型"))?;
    if exists {
        Ok(())
    } else {
        Err(AppError::NotFound("模型不存在".to_owned()))
    }
}

async fn ensure_model_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    model_id: Uuid,
) -> AppResult<()> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM global_model_catalog \
         WHERE id=$1 AND deleted_at IS NULL)",
    )
    .bind(model_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage("无法核验模型"))?;
    if exists {
        Ok(())
    } else {
        Err(AppError::NotFound("模型不存在".to_owned()))
    }
}
