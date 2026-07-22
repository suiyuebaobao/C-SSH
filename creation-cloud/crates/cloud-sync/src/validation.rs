//! 在数据库事务前完成同步白名单、类型、长度和敏感字段校验。

use std::collections::HashSet;

use cloud_domain::{AppError, AppResult};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::types::{
    PullMode, PullRequest, PushRequest, ResolveConflictRequest, SyncChange, SyncOperation,
};

pub(crate) const MAX_CHANGES: usize = 100;
const MAX_VALUE_BYTES: usize = 16 * 1024;

pub(crate) fn push(request: &PushRequest) -> AppResult<()> {
    if request.base_revision < 0 {
        return Err(AppError::Validation("base_revision 不能为负数".to_owned()));
    }
    if request.client_mutation_id.is_nil() {
        return Err(AppError::Validation(
            "client_mutation_id 不能为空".to_owned(),
        ));
    }
    change_set(&request.changes)
}

pub(crate) fn resolve(request: &ResolveConflictRequest) -> AppResult<()> {
    if request.resolution_mutation_id().is_nil() {
        return Err(AppError::Validation(
            "resolution_mutation_id 不能为空".to_owned(),
        ));
    }
    if let Some(changes) = request.changes() {
        change_set(changes)?;
    }
    Ok(())
}

pub(crate) fn change_set(changes: &[SyncChange]) -> AppResult<()> {
    if changes.is_empty() || changes.len() > MAX_CHANGES {
        return Err(AppError::Validation(format!(
            "changes 数量必须在 1 到 {MAX_CHANGES} 之间"
        )));
    }

    let mut identities = HashSet::with_capacity(changes.len());
    for change in changes {
        let identity = (change.namespace.as_str(), change.key.as_str());
        if !identities.insert(identity) {
            return Err(AppError::Validation(
                "同一次 mutation 不得重复修改同一同步键".to_owned(),
            ));
        }
        change_value(change)?;
    }
    Ok(())
}

pub(crate) fn pull(request: PullRequest) -> AppResult<PullRequest> {
    if request.since_revision < 0 {
        return Err(AppError::Validation("since_revision 不能为负数".to_owned()));
    }
    if !(1..=200).contains(&request.limit) {
        return Err(AppError::Validation(
            "limit 必须在 1 到 200 之间".to_owned(),
        ));
    }
    match request.mode {
        PullMode::Incremental if request.snapshot_revision.is_some() => {
            return Err(AppError::Validation(
                "增量拉取不得携带 snapshot_revision".to_owned(),
            ));
        }
        PullMode::Full if request.since_revision == 0 && request.snapshot_revision.is_some() => {
            return Err(AppError::Validation(
                "全量首个分页必须由服务端锁定 snapshot_revision".to_owned(),
            ));
        }
        PullMode::Full if request.since_revision > 0 && request.snapshot_revision.is_none() => {
            return Err(AppError::Validation(
                "全量后续分页必须携带 snapshot_revision".to_owned(),
            ));
        }
        PullMode::Full
            if request
                .snapshot_revision
                .is_some_and(|snapshot| snapshot < request.since_revision) =>
        {
            return Err(AppError::Validation(
                "snapshot_revision 不能早于全量分页游标".to_owned(),
            ));
        }
        _ => {}
    }
    Ok(request)
}

pub(crate) fn conflict_id(conflict_id: Uuid) -> AppResult<()> {
    if conflict_id.is_nil() {
        return Err(AppError::Validation("冲突标识不能为空".to_owned()));
    }
    Ok(())
}

fn change_value(change: &SyncChange) -> AppResult<()> {
    match change.operation {
        SyncOperation::Delete => {
            if change.value.is_some() {
                return Err(AppError::Validation(
                    "删除同步项时不得携带 value".to_owned(),
                ));
            }
            validate_identity(&change.namespace, &change.key)
        }
        SyncOperation::Upsert => {
            let value = change
                .value
                .as_ref()
                .ok_or_else(|| AppError::Validation("写入同步项时必须携带 value".to_owned()))?;
            validate_identity(&change.namespace, &change.key)?;
            reject_sensitive_fields(value)?;
            let encoded = serde_json::to_vec(value)
                .map_err(|_| AppError::Validation("同步 JSON 无法编码".to_owned()))?;
            if encoded.len() > MAX_VALUE_BYTES {
                return Err(AppError::Validation("同步 JSON 超过 16 KiB".to_owned()));
            }
            validate_schema(&change.namespace, &change.key, value.clone())
        }
    }
}

fn validate_identity(namespace: &str, key: &str) -> AppResult<()> {
    if matches!(
        (namespace, key),
        ("appearance", "preferences")
            | ("performance", "defaults")
            | ("terminal", "preferences")
            | ("ui", "template")
    ) {
        return Ok(());
    }
    Err(AppError::Validation(
        "同步 namespace/key 不在白名单中".to_owned(),
    ))
}

fn validate_schema(namespace: &str, key: &str, value: Value) -> AppResult<()> {
    let valid = match (namespace, key) {
        ("appearance", "preferences") => serde_json::from_value::<Appearance>(value)
            .is_ok_and(|preferences| preferences.is_valid()),
        ("performance", "defaults") => serde_json::from_value::<Performance>(value)
            .is_ok_and(|preferences| preferences.is_valid()),
        ("terminal", "preferences") => serde_json::from_value::<Terminal>(value)
            .is_ok_and(|preferences| preferences.is_valid()),
        ("ui", "template") => {
            serde_json::from_value::<UiTemplate>(value).is_ok_and(|template| template.is_valid())
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AppError::Validation(
            "同步 JSON 字段或值不符合白名单".to_owned(),
        ))
    }
}

fn reject_sensitive_fields(value: &Value) -> AppResult<()> {
    match value {
        Value::Object(entries) => {
            for (key, nested) in entries {
                let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                if is_forbidden_key(&normalized) {
                    return Err(AppError::Validation(format!(
                        "同步字段 {key} 属于禁止上云内容"
                    )));
                }
                reject_sensitive_fields(nested)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_sensitive_fields(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn is_forbidden_key(key: &str) -> bool {
    [
        "host",
        "hostname",
        "address",
        "server",
        "ssh",
        "username",
        "password",
        "credential",
        "privatekey",
        "token",
        "apikey",
        "authorization",
        "command",
        "terminalcontent",
        "path",
        "filecontent",
        "aiprompt",
        "airesponse",
        "conversation",
    ]
    .iter()
    .any(|blocked| key.contains(blocked))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Appearance {
    locale: Locale,
    theme: Theme,
    layout: Layout,
    visible_columns: Vec<VisibleColumn>,
}

impl Appearance {
    fn is_valid(&self) -> bool {
        let _ = (&self.locale, &self.theme, &self.layout);
        !self.visible_columns.is_empty() && self.visible_columns.len() <= 8
    }
}

#[derive(Deserialize)]
enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Layout {
    Comfortable,
    Compact,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum VisibleColumn {
    Name,
    Status,
    Platform,
    LastSeen,
    Latency,
    Cpu,
    Memory,
    Disk,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Performance {
    request_profile: RequestProfile,
    monitor_concurrency: u8,
}

impl Performance {
    fn is_valid(&self) -> bool {
        let _ = &self.request_profile;
        (1..=10).contains(&self.monitor_concurrency)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum RequestProfile {
    Stable,
    Balanced,
    Fast,
    Ultra,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Terminal {
    font_family: FontFamily,
    font_size: u8,
    size_mode: SizeMode,
    scrollback: u32,
}

impl Terminal {
    fn is_valid(&self) -> bool {
        let _ = (&self.font_family, &self.size_mode);
        (8..=32).contains(&self.font_size) && (100..=100_000).contains(&self.scrollback)
    }
}

#[derive(Deserialize)]
enum FontFamily {
    #[serde(rename = "system-monospace")]
    SystemMonospace,
    Consolas,
    #[serde(rename = "Cascadia Mono")]
    CascadiaMono,
    #[serde(rename = "JetBrains Mono")]
    JetBrainsMono,
    Menlo,
    #[serde(rename = "monospace")]
    Monospace,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SizeMode {
    Fit,
    Fixed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UiTemplate {
    density: Layout,
    show_toolbar: bool,
    show_status: bool,
}

impl UiTemplate {
    fn is_valid(&self) -> bool {
        let _ = (&self.density, self.show_toolbar, self.show_status);
        true
    }
}
