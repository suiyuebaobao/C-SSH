//! 全局模型目录的公开契约。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGlobalModelInput {
    pub name: String,
    pub provider: String,
    pub context_length: i32,
    pub interfaces: Vec<ModelInterface>,
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
    pub context_length: i32,
    pub interfaces: Vec<ModelInterface>,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelInterface {
    pub api_format: String,
    pub base_url: String,
    pub model_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GlobalModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub context_length: i32,
    pub interfaces: Vec<ModelInterface>,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedModel {
    pub name: String,
    pub provider: String,
    pub context_length: i32,
    pub interfaces: ValidatedInterfaces,
    pub capability_tags: Vec<String>,
    pub default_parameters: Value,
    pub enabled: bool,
    pub is_default: bool,
    pub sort_order: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedInterfaces {
    pub openai_base_url: Option<String>,
    pub openai_model_name: Option<String>,
    pub anthropic_base_url: Option<String>,
    pub anthropic_model_name: Option<String>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct PersistedModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub openai_base_url: Option<String>,
    pub openai_model_name: Option<String>,
    pub anthropic_base_url: Option<String>,
    pub anthropic_model_name: Option<String>,
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

#[derive(Clone, Debug, Serialize)]
pub struct PublicGlobalModel {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub context_length: i32,
    pub interfaces: Vec<ModelInterface>,
    pub enabled: bool,
    pub revision: i64,
    pub updated_at: DateTime<Utc>,
}

impl From<GlobalModel> for PublicGlobalModel {
    fn from(value: GlobalModel) -> Self {
        Self {
            id: value.id,
            name: value.name,
            provider: value.provider,
            context_length: value.context_length,
            interfaces: value.interfaces,
            enabled: value.enabled,
            revision: value.revision,
            updated_at: value.updated_at,
        }
    }
}

impl From<PersistedModel> for GlobalModel {
    fn from(value: PersistedModel) -> Self {
        let mut interfaces = Vec::with_capacity(2);
        if let (Some(base_url), Some(model_name)) = (value.openai_base_url, value.openai_model_name)
        {
            interfaces.push(ModelInterface {
                api_format: "openai_compatible".to_owned(),
                base_url,
                model_name,
            });
        }
        if let (Some(base_url), Some(model_name)) =
            (value.anthropic_base_url, value.anthropic_model_name)
        {
            interfaces.push(ModelInterface {
                api_format: "anthropic_compatible".to_owned(),
                base_url,
                model_name,
            });
        }
        Self {
            id: value.id,
            name: value.name,
            provider: value.provider,
            context_length: value.context_length,
            interfaces,
            capability_tags: value.capability_tags,
            default_parameters: value.default_parameters,
            enabled: value.enabled,
            is_default: value.is_default,
            sort_order: value.sort_order,
            revision: value.revision,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<CreateGlobalModelInput> for ReplaceGlobalModelInput {
    fn from(value: CreateGlobalModelInput) -> Self {
        Self {
            expected_revision: 1,
            name: value.name,
            provider: value.provider,
            context_length: value.context_length,
            interfaces: value.interfaces,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn public_json_uses_interfaces_and_has_no_singular_endpoint_fields() {
        let model = GlobalModel {
            id: Uuid::now_v7(),
            name: "Kimi K3".to_owned(),
            provider: "Kimi".to_owned(),
            context_length: 1_048_576,
            interfaces: vec![
                ModelInterface {
                    api_format: "openai_compatible".to_owned(),
                    base_url: "https://api.moonshot.cn/v1".to_owned(),
                    model_name: "kimi-k3".to_owned(),
                },
                ModelInterface {
                    api_format: "anthropic_compatible".to_owned(),
                    base_url: "https://api.moonshot.cn/anthropic".to_owned(),
                    model_name: "kimi-k3[1m]".to_owned(),
                },
            ],
            capability_tags: Vec::new(),
            default_parameters: json!({}),
            enabled: true,
            is_default: false,
            sort_order: 10,
            revision: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let value = serde_json::to_value(PublicGlobalModel::from(model)).expect("公开模型应可编码");
        assert_eq!(value["interfaces"].as_array().map(Vec::len), Some(2));
        assert_eq!(value["interfaces"][1]["model_name"], "kimi-k3[1m]");
        assert!(value.get("api_format").is_none());
        assert!(value.get("base_url").is_none());
        assert!(value.get("model_name").is_none());
        assert!(value.get("api_key").is_none());
        assert!(value.get("is_default").is_none());
        assert!(value.get("sort_order").is_none());
        assert!(value.get("default_parameters").is_none());
        assert!(value.get("capability_tags").is_none());
        assert!(value.get("created_at").is_none());
    }
}
