//! 全局模型目录的公开契约。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReasoningControl {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "glm")]
    Glm,
    #[serde(rename = "qwen")]
    Qwen,
    #[serde(rename = "kimi")]
    Kimi,
    #[serde(rename = "minimax")]
    MiniMax,
    #[default]
    #[serde(rename = "unsupported")]
    Unsupported,
}

impl ReasoningControl {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::Glm => "glm",
            Self::Qwen => "qwen",
            Self::Kimi => "kimi",
            Self::MiniMax => "minimax",
            Self::Unsupported => "unsupported",
        }
    }

    fn from_persisted(value: &str) -> Self {
        match value {
            "openai" => Self::OpenAi,
            "deepseek" => Self::DeepSeek,
            "glm" => Self::Glm,
            "qwen" => Self::Qwen,
            "kimi" => Self::Kimi,
            "minimax" => Self::MiniMax,
            "unsupported" => Self::Unsupported,
            _ => unreachable!("reasoning_control 由数据库 CHECK 约束"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateGlobalModelInput {
    pub name: String,
    pub provider: String,
    pub context_length: i32,
    pub interfaces: Vec<ModelInterface>,
    #[serde(default)]
    pub reasoning_control: ReasoningControl,
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
    pub reasoning_control: ReasoningControl,
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
    pub reasoning_control: ReasoningControl,
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
    pub reasoning_control: ReasoningControl,
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
    pub responses_base_url: Option<String>,
    pub responses_model_name: Option<String>,
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
    pub responses_base_url: Option<String>,
    pub responses_model_name: Option<String>,
    pub reasoning_control: String,
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
    pub reasoning_control: ReasoningControl,
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
            // 该字段只为旧客户端反序列化兼容保留；接口格式才是运行时契约。
            reasoning_control: ReasoningControl::Unsupported,
            enabled: value.enabled,
            revision: value.revision,
            updated_at: value.updated_at,
        }
    }
}

impl From<PersistedModel> for GlobalModel {
    fn from(value: PersistedModel) -> Self {
        let mut interfaces = Vec::with_capacity(3);
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
        if let (Some(base_url), Some(model_name)) =
            (value.responses_base_url, value.responses_model_name)
        {
            interfaces.push(ModelInterface {
                api_format: "responses_compatible".to_owned(),
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
            reasoning_control: ReasoningControl::from_persisted(&value.reasoning_control),
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
            reasoning_control: value.reasoning_control,
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
    fn legacy_reasoning_control_wire_values_remain_deserializable() {
        for (raw, expected) in [
            ("unsupported", ReasoningControl::Unsupported),
            ("openai", ReasoningControl::OpenAi),
            ("deepseek", ReasoningControl::DeepSeek),
            ("glm", ReasoningControl::Glm),
            ("qwen", ReasoningControl::Qwen),
            ("kimi", ReasoningControl::Kimi),
            ("minimax", ReasoningControl::MiniMax),
        ] {
            assert_eq!(
                serde_json::from_value::<ReasoningControl>(json!(raw)).expect("受信思考类型应有效"),
                expected
            );
            assert_eq!(expected.as_str(), raw);
        }
        assert!(serde_json::from_value::<ReasoningControl>(json!("supported")).is_err());
        assert!(serde_json::from_value::<ReasoningControl>(json!({})).is_err());
    }

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
                ModelInterface {
                    api_format: "responses_compatible".to_owned(),
                    base_url: "https://api.moonshot.cn/v1".to_owned(),
                    model_name: "kimi-k3".to_owned(),
                },
            ],
            reasoning_control: ReasoningControl::Kimi,
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
        assert_eq!(value["interfaces"].as_array().map(Vec::len), Some(3));
        assert_eq!(value["interfaces"][1]["model_name"], "kimi-k3[1m]");
        assert_eq!(value["interfaces"][2]["api_format"], "responses_compatible");
        assert_eq!(value["reasoning_control"], "unsupported");
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
