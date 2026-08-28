//! 全局模型目录查询与管理员 CAS 写入。

use cloud_domain::{AppError, AppResult, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    GlobalModel,
    types::{PersistedModel, ValidatedModel},
};

use super::storage;

const MODEL_COLUMNS: &str = "id, name, provider, openai_base_url, openai_model_name, \
    anthropic_base_url, anthropic_model_name, responses_base_url, responses_model_name, \
    reasoning_control, context_length, capability_tags, default_parameters, enabled, \
    is_default, sort_order, revision, created_at, updated_at";

pub(crate) async fn list_public(pool: &PgPool, page: PageQuery) -> AppResult<Page<GlobalModel>> {
    list(pool, page, true).await
}

pub(crate) async fn list_admin(pool: &PgPool, page: PageQuery) -> AppResult<Page<GlobalModel>> {
    list(pool, page, false).await
}

async fn list(pool: &PgPool, page: PageQuery, public_only: bool) -> AppResult<Page<GlobalModel>> {
    let page = page.normalized();
    let filter = if public_only {
        "deleted_at IS NULL AND enabled"
    } else {
        "deleted_at IS NULL"
    };
    let total_sql = format!("SELECT COUNT(*) FROM global_model_catalog WHERE {filter}");
    let total = sqlx::query_scalar::<_, i64>(&total_sql)
        .fetch_one(pool)
        .await
        .map_err(storage("无法统计模型目录"))?;
    let sql = format!(
        "SELECT {MODEL_COLUMNS} FROM global_model_catalog WHERE {filter} \
         ORDER BY is_default DESC, sort_order ASC, name ASC, id ASC LIMIT $1 OFFSET $2"
    );
    let items = sqlx::query_as::<_, PersistedModel>(&sql)
        .bind(i64::from(page.size))
        .bind(page.offset())
        .fetch_all(pool)
        .await
        .map_err(storage("无法读取模型目录"))?
        .into_iter()
        .map(GlobalModel::from)
        .collect();
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn get_public(pool: &PgPool, id: Uuid) -> AppResult<GlobalModel> {
    get(pool, id, true).await
}

pub(crate) async fn get_admin(pool: &PgPool, id: Uuid) -> AppResult<GlobalModel> {
    get(pool, id, false).await
}

async fn get(pool: &PgPool, id: Uuid, public_only: bool) -> AppResult<GlobalModel> {
    let filter = if public_only {
        "id = $1 AND deleted_at IS NULL AND enabled"
    } else {
        "id = $1 AND deleted_at IS NULL"
    };
    let sql = format!("SELECT {MODEL_COLUMNS} FROM global_model_catalog WHERE {filter}");
    sqlx::query_as::<_, PersistedModel>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(storage("无法读取模型"))?
        .ok_or_else(|| AppError::NotFound("模型不存在".to_owned()))
        .map(GlobalModel::from)
}

pub(crate) async fn create(
    pool: &PgPool,
    actor_id: Uuid,
    value: ValidatedModel,
) -> AppResult<GlobalModel> {
    let mut tx = pool
        .begin()
        .await
        .map_err(storage("无法开始模型创建事务"))?;
    lock_catalog(&mut tx).await?;
    if value.is_default {
        clear_default(&mut tx, None, actor_id).await?;
    }
    let id = Uuid::now_v7();
    let sql = format!(
        "INSERT INTO global_model_catalog \
         (id, name, provider, openai_base_url, openai_model_name, anthropic_base_url, \
          anthropic_model_name, responses_base_url, responses_model_name, reasoning_control, \
          context_length, capability_tags, default_parameters, enabled, is_default, sort_order, \
          created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$17) \
         RETURNING {MODEL_COLUMNS}"
    );
    let model = sqlx::query_as::<_, PersistedModel>(&sql)
        .bind(id)
        .bind(value.name)
        .bind(value.provider)
        .bind(value.interfaces.openai_base_url)
        .bind(value.interfaces.openai_model_name)
        .bind(value.interfaces.anthropic_base_url)
        .bind(value.interfaces.anthropic_model_name)
        .bind(value.interfaces.responses_base_url)
        .bind(value.interfaces.responses_model_name)
        .bind(value.reasoning_control.as_str())
        .bind(value.context_length)
        .bind(value.capability_tags)
        .bind(value.default_parameters)
        .bind(value.enabled)
        .bind(value.is_default)
        .bind(value.sort_order)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage("无法创建全局模型"))?;
    tx.commit().await.map_err(storage("无法提交模型创建事务"))?;
    Ok(model.into())
}

pub(crate) async fn replace(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
    value: ValidatedModel,
) -> AppResult<GlobalModel> {
    let mut tx = pool
        .begin()
        .await
        .map_err(storage("无法开始模型更新事务"))?;
    lock_catalog(&mut tx).await?;
    require_revision(&mut tx, id, expected_revision).await?;
    if value.is_default {
        clear_default(&mut tx, Some(id), actor_id).await?;
    }
    let sql = format!(
        "UPDATE global_model_catalog SET name=$2, provider=$3, openai_base_url=$4, \
         openai_model_name=$5, anthropic_base_url=$6, anthropic_model_name=$7, \
         responses_base_url=$8, responses_model_name=$9, reasoning_control=$10, \
         context_length=$11, capability_tags=$12, default_parameters=$13, enabled=$14, \
         is_default=$15, sort_order=$16, revision=revision+1, updated_by=$17, updated_at=now() \
         WHERE id=$1 AND deleted_at IS NULL RETURNING {MODEL_COLUMNS}"
    );
    let model = sqlx::query_as::<_, PersistedModel>(&sql)
        .bind(id)
        .bind(value.name)
        .bind(value.provider)
        .bind(value.interfaces.openai_base_url)
        .bind(value.interfaces.openai_model_name)
        .bind(value.interfaces.anthropic_base_url)
        .bind(value.interfaces.anthropic_model_name)
        .bind(value.interfaces.responses_base_url)
        .bind(value.interfaces.responses_model_name)
        .bind(value.reasoning_control.as_str())
        .bind(value.context_length)
        .bind(value.capability_tags)
        .bind(value.default_parameters)
        .bind(value.enabled)
        .bind(value.is_default)
        .bind(value.sort_order)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage("无法更新全局模型"))?;
    tx.commit().await.map_err(storage("无法提交模型更新事务"))?;
    Ok(model.into())
}

pub(crate) async fn delete(
    pool: &PgPool,
    actor_id: Uuid,
    id: Uuid,
    expected_revision: i64,
) -> AppResult<()> {
    let mut tx = pool
        .begin()
        .await
        .map_err(storage("无法开始模型删除事务"))?;
    lock_catalog(&mut tx).await?;
    require_revision(&mut tx, id, expected_revision).await?;
    sqlx::query(
        "UPDATE global_model_catalog SET enabled=FALSE, is_default=FALSE, \
         revision=revision+1, updated_by=$2, updated_at=now(), deleted_at=now() \
         WHERE id=$1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(actor_id)
    .execute(&mut *tx)
    .await
    .map_err(storage("无法删除全局模型"))?;
    tx.commit().await.map_err(storage("无法提交模型删除事务"))?;
    Ok(())
}

async fn lock_catalog(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> AppResult<()> {
    sqlx::query("LOCK TABLE global_model_catalog IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut **tx)
        .await
        .map_err(storage("无法锁定模型目录"))?;
    Ok(())
}

async fn require_revision(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    expected: i64,
) -> AppResult<()> {
    let current = sqlx::query_scalar::<_, i64>(
        "SELECT revision FROM global_model_catalog \
         WHERE id=$1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage("无法锁定模型"))?
    .ok_or_else(|| AppError::NotFound("模型不存在".to_owned()))?;
    if current != expected {
        return Err(AppError::Conflict("模型 revision 已变化".to_owned()));
    }
    Ok(())
}

async fn clear_default(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    except: Option<Uuid>,
    actor_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE global_model_catalog SET is_default=FALSE, revision=revision+1, \
         updated_by=$2, updated_at=now() WHERE deleted_at IS NULL AND is_default \
         AND ($1::uuid IS NULL OR id <> $1)",
    )
    .bind(except)
    .bind(actor_id)
    .execute(&mut **tx)
    .await
    .map_err(storage("无法切换默认模型"))?;
    Ok(())
}
