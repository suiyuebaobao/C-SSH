//! 组装“已上传”页的版本与文件摘要，不复制发布或资产状态机。

use cloud_domain::{AdminActor, formal_release_asset_identities};
use cloud_download::SourceKind;
use cloud_release::{Release, ReleaseAsset};

use crate::AdminPageState;

pub(super) struct ReleaseFileRow {
    pub(super) platform: String,
    pub(super) architecture: String,
    pub(super) package_kind: String,
    pub(super) file_name: String,
    pub(super) byte_size: String,
    pub(super) sha256: String,
    pub(super) source_ready: bool,
}

pub(super) struct ReleaseRow {
    pub(super) id: String,
    pub(super) version: String,
    pub(super) channel: &'static str,
    pub(super) status: &'static str,
    pub(super) title_zh: String,
    pub(super) title_en: String,
    pub(super) notes_zh: String,
    pub(super) notes_en: String,
    pub(super) published_at: String,
    pub(super) updated_at: String,
    pub(super) files: Vec<ReleaseFileRow>,
    pub(super) files_error: bool,
    pub(super) ready_for_validation: bool,
}

impl ReleaseRow {
    async fn load(state: &AdminPageState, actor: &AdminActor, release: Release) -> Self {
        let assets = state.release().list_assets(actor, release.id).await;
        let mut files = Vec::new();
        let mut files_error = assets.is_err();
        if let Ok(assets) = assets {
            for asset in assets {
                let sources = state.download().list_sources(actor, asset.id).await;
                let source_ready = match sources {
                    Ok(sources) => sources.iter().any(|source| {
                        source.enabled
                            && match source.source_kind {
                                SourceKind::Local => source.local_path.is_some(),
                                SourceKind::External => source.external_url.is_some(),
                            }
                    }),
                    Err(_) => {
                        files_error = true;
                        false
                    }
                };
                files.push(ReleaseFileRow::new(asset, source_ready));
            }
        }
        let ready_for_validation = !files_error && has_complete_asset_set(&release.version, &files);
        Self {
            id: release.id.to_string(),
            version: release.version,
            channel: release.channel.as_str(),
            status: release.status.as_str(),
            title_zh: release.title_zh,
            title_en: release.title_en,
            notes_zh: release.notes_zh,
            notes_en: release.notes_en,
            published_at: release
                .published_at
                .map_or_else(|| "—".to_owned(), |at| at.to_rfc3339()),
            updated_at: release.updated_at.to_rfc3339(),
            files,
            files_error,
            ready_for_validation,
        }
    }
}

impl ReleaseFileRow {
    fn new(asset: ReleaseAsset, source_ready: bool) -> Self {
        Self {
            platform: asset.platform,
            architecture: asset.architecture,
            package_kind: asset.package_kind,
            file_name: asset.file_name,
            byte_size: format_bytes(asset.byte_size),
            sha256: asset.sha256,
            source_ready,
        }
    }
}

pub(super) async fn load_rows(
    state: &AdminPageState,
    actor: &AdminActor,
    releases: Vec<Release>,
) -> Vec<ReleaseRow> {
    let mut rows = Vec::with_capacity(releases.len());
    for release in releases {
        rows.push(ReleaseRow::load(state, actor, release).await);
    }
    rows
}

fn has_complete_asset_set(version: &str, files: &[ReleaseFileRow]) -> bool {
    let Some(expected) = formal_release_asset_identities(version) else {
        return false;
    };
    files.len() == expected.len()
        && files.iter().all(|file| file.source_ready)
        && expected.iter().all(|expected| {
            files.iter().any(|file| {
                (
                    file.platform.as_str(),
                    file.architecture.as_str(),
                    file.package_kind.as_str(),
                ) == *expected
            })
        })
}

fn format_bytes(value: i64) -> String {
    if value >= 1024 * 1024 * 1024 {
        format!("{:.2} GiB", value as f64 / (1024_f64 * 1024_f64 * 1024_f64))
    } else if value >= 1024 * 1024 {
        format!("{:.2} MiB", value as f64 / (1024_f64 * 1024_f64))
    } else if value >= 1024 {
        format!("{:.2} KiB", value as f64 / 1024_f64)
    } else {
        format!("{value} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(platform: &str, architecture: &str, package_kind: &str) -> ReleaseFileRow {
        ReleaseFileRow {
            platform: platform.to_owned(),
            architecture: architecture.to_owned(),
            package_kind: package_kind.to_owned(),
            file_name: "client.bin".to_owned(),
            byte_size: "1 B".to_owned(),
            sha256: "a".repeat(64),
            source_ready: true,
        }
    }

    #[test]
    fn current_release_is_ready_only_with_the_three_formal_files() {
        let files = vec![
            file("windows", "x86_64", "exe"),
            file("windows", "x86_64", "zip"),
            file("android", "aarch64", "apk"),
        ];
        assert!(has_complete_asset_set("0.8.9", &files));

        let mut incomplete = files;
        incomplete.pop();
        assert!(!has_complete_asset_set("0.8.9", &incomplete));
    }
}
