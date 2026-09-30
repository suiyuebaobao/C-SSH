//! 定义第三方代理线路的不透明密文同步形状，服务端不解析线路名称或 URI。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyProfileOperation {
    Insert,
    Update,
    Delete,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyProfilePayloadInput {
    pub ciphertext: String,
    pub nonce: String,
    pub envelope_metadata: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyProfileChange {
    pub resource_id: Uuid,
    pub operation: ProxyProfileOperation,
    #[serde(default)]
    pub payload: Option<ProxyProfilePayloadInput>,
    #[serde(default)]
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PullProxyProfileRecord {
    pub resource_id: Uuid,
    pub revision: i64,
    pub ciphertext: Option<String>,
    pub nonce: Option<String>,
    pub envelope_metadata: Option<serde_json::Value>,
    pub source_device_id: Uuid,
    pub deleted: bool,
    pub updated_at: DateTime<Utc>,
}
