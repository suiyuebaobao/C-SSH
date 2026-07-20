//! 插入一个始终从草稿状态开始的新版本。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{CreateReleaseInput, Release, model::ReleaseRow, repository::map_write_error};

pub(crate) async fn execute(pool: &PgPool, input: &CreateReleaseInput) -> AppResult<Release> {
    let row = sqlx::query_as::<_, ReleaseRow>(
        r#"
        INSERT INTO releases (
            id, version, channel, status, title_zh, title_en, notes_zh, notes_en
        )
        VALUES ($1, $2, $3, 'draft', $4, $5, $6, $7)
        RETURNING id, version, channel, status, title_zh, title_en,
                  notes_zh, notes_en, published_at, created_at, updated_at
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(&input.version)
    .bind(input.channel.as_str())
    .bind(&input.title_zh)
    .bind(&input.title_en)
    .bind(&input.notes_zh)
    .bind(&input.notes_en)
    .fetch_one(pool)
    .await
    .map_err(|error| map_write_error(error, "版本号已经存在"))?;

    Release::try_from(row)
}
