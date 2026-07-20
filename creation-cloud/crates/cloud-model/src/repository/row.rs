//! 把运行时 query_as 返回的模型行转换为公开领域对象。

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::ModelProfile;

pub(crate) type ModelRow = (
    Uuid,
    String,
    String,
    Option<String>,
    String,
    i32,
    Vec<String>,
    Value,
    bool,
    bool,
    i32,
    Option<Uuid>,
    DateTime<Utc>,
    DateTime<Utc>,
);

pub(crate) fn model_from_row(row: ModelRow) -> ModelProfile {
    ModelProfile {
        id: row.0,
        name: row.1,
        provider: row.2,
        base_url: row.3,
        model_name: row.4,
        context_length: row.5,
        capability_tags: row.6,
        default_parameters: row.7,
        enabled: row.8,
        is_default: row.9,
        sort_order: row.10,
        vault_envelope_id: row.11,
        created_at: row.12,
        updated_at: row.13,
    }
}
