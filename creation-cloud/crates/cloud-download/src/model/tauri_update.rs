//! 定义 Windows Tauri v2 updater 使用的固定平台清单查询与响应。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TauriUpdateQuery {
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
pub(crate) struct TauriUpdateResponse {
    pub version: String,
    pub notes: String,
    pub pub_date: DateTime<Utc>,
    pub platforms: BTreeMap<String, TauriPlatformUpdate>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TauriPlatformUpdate {
    pub url: String,
    pub signature: String,
}

fn default_channel() -> String {
    "stable".to_owned()
}

fn default_locale() -> String {
    "zh-CN".to_owned()
}
