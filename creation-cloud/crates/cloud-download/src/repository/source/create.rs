//! 为一个资产插入独立的本站或外部来源。

use cloud_domain::AppResult;
use cloud_store::{Postgres, Transaction};
use sqlx::Executor;
use uuid::Uuid;

use crate::{CreateSourceInput, ReleaseSource, model::SourceRow, repository::map_write_error};

pub(crate) async fn execute_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    input: &CreateSourceInput,
) -> AppResult<ReleaseSource> {
    insert(&mut **transaction, input).await
}

async fn insert<'executor, E>(executor: E, input: &CreateSourceInput) -> AppResult<ReleaseSource>
where
    E: Executor<'executor, Database = Postgres>,
{
    let row = sqlx::query_as::<_, SourceRow>(
        r#"
        INSERT INTO release_sources (
            id, asset_id, source_kind, provider_name, local_path,
            external_url, sort_order, enabled
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, asset_id, source_kind, provider_name, local_path,
                  external_url, sort_order, enabled, created_at, updated_at
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(input.asset_id)
    .bind(input.source_kind.as_str())
    .bind(&input.provider_name)
    .bind(input.local_path.as_deref())
    .bind(input.external_url.as_deref())
    .bind(input.sort_order)
    .bind(input.enabled)
    .fetch_one(executor)
    .await
    .map_err(|error| map_write_error(error, "下载来源与现有数据冲突"))?;
    ReleaseSource::try_from(row)
}
