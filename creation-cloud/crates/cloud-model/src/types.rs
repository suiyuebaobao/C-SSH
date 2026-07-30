//! 全局模型目录与账号级客户端密文的公开契约。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGlobalModelInput {
    pub name: String,
    pub provider: String,
    #[serde(default = "default_api_format")]
    pub api_format: String,
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
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplaceGlobalModelInput {
    pub expected_revision: i64,
    pub name: String,
    pub provider: String,
    #[serde(default = "default_api_format")]
    pub api_format: String,
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
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteGlobalModelInput {
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct GlobalModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub api_format: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub context_length: i32,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PutModelSecretInput {
    pub ciphertext: String,
    pub expected_revision: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteModelSecretInput {
    pub expected_revision: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModelSecret {
    pub model_id: Uuid,
    pub revision: i64,
    pub present: bool,
    pub ciphertext: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedModel {
    pub name: String,
    pub provider: String,
    pub api_format: String,
    pub base_url: Option<String>,
    pub model_name: String,
    pub context_length: i32,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
}

impl From<CreateGlobalModelInput> for ReplaceGlobalModelInput {
    fn from(value: CreateGlobalModelInput) -> Self {
        Self {
            expected_revision: 1,
            name: value.name,
            provider: value.provider,
            api_format: value.api_format,
            base_url: value.base_url,
            model_name: value.model_name,
            context_length: value.context_length,
            capability_tags: value.capability_tags,
            default_parameters: value.default_parameters,
            enabled: value.enabled,
            is_default: value.is_default,
            sort_order: value.sort_order,
        }
    }
}

fn empty_object() -> Value {
    Value::Object(Map::new())
}

const fn default_true() -> bool {
    true
}

fn default_api_format() -> String {
    "openai_compatible".to_owned()
}
