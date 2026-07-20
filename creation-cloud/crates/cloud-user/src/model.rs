//! 定义用户资料在服务、repository 与 JSON API 间共享的数据模型。

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

pub(crate) type ProfileRow = (Uuid, String, String, DateTime<Utc>, DateTime<Utc>);

#[derive(Clone, Debug, Serialize)]
pub struct Profile {
    pub account_id: Uuid,
    pub display_name: String,
    pub locale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Profile {
    pub(crate) fn from_row(row: ProfileRow) -> Self {
        Self {
            account_id: row.0,
            display_name: row.1,
            locale: row.2,
            created_at: row.3,
            updated_at: row.4,
        }
    }
}
