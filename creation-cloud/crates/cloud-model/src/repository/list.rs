//! 分页读取当前账号的模型元数据列表。

use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::ModelProfile;

use super::{ModelRow, model_from_row, storage};

pub(crate) async fn list(
    pool: &PgPool,
    account_id: Uuid,
    page: PageQuery,
) -> AppResult<Page<ModelProfile>> {
    let total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM model_profiles WHERE account_id = $1")
            .bind(account_id)
            .fetch_one(pool)
            .await
            .map_err(storage("无法统计模型元数据"))?;
    let rows = sqlx::query_as::<_, ModelRow>(
        "SELECT id, name, provider, base_url, model_name, context_length, \
         capability_tags, default_parameters, enabled, is_default, sort_order, \
         vault_envelope_id, created_at, updated_at \
         FROM model_profiles WHERE account_id = $1 \
         ORDER BY is_default DESC, sort_order ASC, name ASC, id ASC LIMIT $2 OFFSET $3",
    )
    .bind(account_id)
    .bind(i64::from(page.size))
    .bind(page.offset())
    .fetch_all(pool)
    .await
    .map_err(storage("无法读取模型列表"))?;
    Ok(Page {
        items: rows.into_iter().map(model_from_row).collect(),
        page: page.page,
        size: page.size,
        total,
    })
}
