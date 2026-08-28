//! 定义受控发布编排写入版本化正式资产安装身份的严格 JSON 合同。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledIdentityEntryInput {
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub download_sha256: String,
    pub installed_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RecordInstalledIdentitiesInput {
    pub release_id: Uuid,
    pub version: String,
    pub expected_release_updated_at: String,
    pub product_input_sha256: String,
    pub entries: Vec<InstalledIdentityEntryInput>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecordInstalledIdentitiesResult {
    pub release_id: Uuid,
    pub version: String,
    pub release_updated_at: String,
    pub product_input_sha256: String,
    pub entry_count: usize,
    pub idempotent: bool,
}
