//! 定义兼容现有客户端的匿名更新响应，并追加强制策略与签名元数据。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::SourceKind;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCheckQuery {
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub current_version: String,
    pub current_install_sha256: Option<String>,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateCheckResponse {
    pub update_available: bool,
    pub current_version: String,
    pub latest: Option<LatestUpdate>,
    pub required: bool,
    pub policy_revision: Option<u64>,
    pub sha256_enabled: bool,
    pub identity_status: UpdateIdentityStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateIdentityStatus {
    NotApplicable,
    Verified,
    Missing,
    Mismatch,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatestUpdate {
    pub version: String,
    pub channel: String,
    pub title: String,
    pub notes: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<UpdateAsset>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateAsset {
    pub id: Uuid,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub updater_signature: Option<String>,
    pub sources: Vec<UpdateSource>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateSource {
    pub source_kind: SourceKind,
    pub provider_name: String,
    pub download_url: String,
}

fn default_channel() -> String {
    "stable".to_owned()
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_extends_the_existing_latest_contract_without_replacing_it() {
        let value = serde_json::to_value(UpdateCheckResponse {
            update_available: true,
            current_version: "0.7.7".into(),
            latest: Some(LatestUpdate {
                version: "0.8.0".into(),
                channel: "stable".into(),
                title: "Creation-SSH 0.8.0".into(),
                notes: "notes".into(),
                published_at: Utc::now(),
                assets: vec![UpdateAsset {
                    id: Uuid::now_v7(),
                    architecture: "x86_64".into(),
                    package_kind: "exe".into(),
                    file_name: "Creation-SSH.exe".into(),
                    byte_size: 1,
                    sha256: "a".repeat(64),
                    updater_signature: Some("b".repeat(64)),
                    sources: vec![UpdateSource {
                        source_kind: SourceKind::Local,
                        provider_name: "本站".into(),
                        download_url: "/api/v1/downloads/assets/a/sources/b".into(),
                    }],
                }],
            }),
            required: true,
            policy_revision: Some(3),
            sha256_enabled: true,
            identity_status: UpdateIdentityStatus::Verified,
        })
        .expect("更新响应应可序列化");
        for key in [
            "update_available",
            "current_version",
            "latest",
            "required",
            "policy_revision",
            "sha256_enabled",
            "identity_status",
        ] {
            assert!(value.get(key).is_some(), "缺少兼容字段 {key}");
        }
        assert_eq!(value["latest"]["assets"][0]["sha256"], "a".repeat(64));
        assert!(value["latest"]["assets"][0]["updater_signature"].is_string());
        assert_eq!(value["identity_status"], "verified");
        assert!(value.get("expected_sha256").is_none());
        assert!(value.get("installed_sha256").is_none());
    }
}
