//! 定义模型元数据 API、持久化命令与公开响应对象。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct CreateModelInput {
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub context_length: i32,
    #[serde(default)]
    pub capability_tags: Vec<String>,
    #[serde(default = "empty_object")]
    pub default_parameters: Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i32,
    pub vault_envelope_id: Option<Uuid>,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateModelInput {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub model_name: Option<String>,
    pub context_length: Option<i32>,
    pub capability_tags: Option<Vec<String>>,
    pub default_parameters: Option<Value>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
    pub sort_order: Option<i32>,
    pub vault_envelope_id: Option<Uuid>,
    #[serde(default)]
    pub clear_vault_envelope: bool,
    #[serde(default, flatten)]
    pub extra_fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModelProfile {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub context_length: i32,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub vault_envelope_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) struct CreateModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub context_length: i32,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub vault_envelope_id: Option<Uuid>,
}

pub(crate) struct UpdateModel {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<Option<String>>,
    pub model_name: Option<String>,
    pub context_length: Option<i32>,
    pub capability_tags: Option<Vec<String>>,
    pub default_parameters: Option<Value>,
    pub enabled: Option<bool>,
    pub is_default: Option<bool>,
    pub sort_order: Option<i32>,
    pub vault_envelope_id: Option<Option<Uuid>>,
}

fn empty_object() -> Value {
    Value::Object(Map::new())
}

const fn default_true() -> bool {
    true
}
