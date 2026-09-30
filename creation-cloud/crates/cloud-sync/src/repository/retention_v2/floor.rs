//! 汇总资源清理水位并单调推进账号统一同步压缩边界。

use crate::repository::storage;
use cloud_domain::AppResult;
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) async fn advance<'a>(
    transaction: &mut Transaction<'_, Postgres>,
    rows: impl Iterator<Item = &'a (Uuid, i64)>,
) -> AppResult<()> {
    let mut floors = BTreeMap::<Uuid, i64>::new();
    for (account_id, revision) in rows {
        floors
            .entry(*account_id)
            .and_modify(|current| *current = (*current).max(*revision))
            .or_insert(*revision);
    }
    for (account_id, revision) in floors {
        sqlx::query(super::ADVANCE_FLOOR_SQL)
            .bind(account_id)
            .bind(revision)
            .execute(&mut **transaction)
            .await
            .map_err(storage("无法推进统一密文同步压缩边界"))?;
    }
    Ok(())
}
