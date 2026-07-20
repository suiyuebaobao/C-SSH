//! 把运行时 query_as 返回的同步行转换为公开领域对象。

use chrono::{DateTime, Utc};
use cloud_domain::{AppError, AppResult};
use serde_json::Value;
use uuid::Uuid;

use crate::{SyncChange, SyncConflict};

pub(crate) type ConflictRow = (Uuid, Uuid, i64, i64, Value, DateTime<Utc>);

pub(crate) fn conflict_from_row(row: ConflictRow) -> AppResult<SyncConflict> {
    let attempted_changes = serde_json::from_value::<Vec<SyncChange>>(row.4)
        .map_err(|_| AppError::Internal("同步冲突记录格式无效".to_owned()))?;
    Ok(SyncConflict {
        id: row.0,
        client_mutation_id: row.1,
        base_revision: row.2,
        current_revision: row.3,
        attempted_changes,
        created_at: row.5,
    })
}
