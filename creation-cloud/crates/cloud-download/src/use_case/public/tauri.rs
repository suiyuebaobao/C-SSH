//! 从同一策略检查结果投影 Windows Tauri清单，确保版本、资产和revision不分叉。

use std::collections::BTreeMap;

use cloud_domain::{AppError, AppResult};

use crate::{
    LatestUpdate, Service, UpdateCheckQuery,
    model::{TauriPlatformUpdate, TauriUpdateQuery, TauriUpdateResponse},
};

const CLOUD_ORIGIN: &str = "https://c-ssh.com";

impl Service {
    pub(crate) async fn tauri_update(
        &self,
        query: TauriUpdateQuery,
    ) -> AppResult<Option<TauriUpdateResponse>> {
        validate_query(&query)?;
        let result = self.check_update(shared_check_query(&query)).await?;
        if !result.update_available {
            return Ok(None);
        }
        let latest = result
            .latest
            .ok_or_else(|| AppError::Storage("更新检查结果缺少 latest".into()))?;
        project_response(latest, &query.package_kind, &query.architecture).map(Some)
    }
}

fn shared_check_query(query: &TauriUpdateQuery) -> UpdateCheckQuery {
    UpdateCheckQuery {
        platform: "windows".into(),
        architecture: query.architecture.clone(),
        package_kind: query.package_kind.clone(),
        current_version: query.current_version.clone(),
        current_install_sha256: query.current_install_sha256.clone(),
        channel: query.channel.clone(),
        locale: query.locale.clone(),
    }
}

fn project_response(
    latest: LatestUpdate,
    package_kind: &str,
    architecture: &str,
) -> AppResult<TauriUpdateResponse> {
    let asset = latest
        .assets
        .into_iter()
        .find(|asset| asset.package_kind == package_kind)
        .ok_or_else(|| AppError::Storage("Tauri 清单缺少匹配安装资产".into()))?;
    let signature = asset
        .updater_signature
        .as_deref()
        .ok_or_else(|| AppError::Storage("Tauri 安装资产缺少 signature".into()))?;
    let signature = crate::signature::validate(signature)?;
    let source = asset
        .sources
        .first()
        .ok_or_else(|| AppError::Storage("Tauri 安装资产缺少本站来源".into()))?;
    if !source.download_url.starts_with("/api/v1/downloads/assets/")
        || source.download_url.contains(['?', '#'])
    {
        return Err(AppError::Storage("Tauri 下载入口不属于本站固定路径".into()));
    }
    let target = format!("windows-{architecture}");
    let platforms = BTreeMap::from([(
        target,
        TauriPlatformUpdate {
            url: format!("{CLOUD_ORIGIN}{}", source.download_url),
            signature,
        },
    )]);
    Ok(TauriUpdateResponse {
        version: latest.version,
        notes: latest.notes,
        pub_date: latest.published_at,
        platforms,
    })
}

fn validate_query(query: &TauriUpdateQuery) -> AppResult<()> {
    if query.architecture != "x86_64" {
        return Err(AppError::Validation("Tauri 更新架构只允许 x86_64".into()));
    }
    if !matches!(query.package_kind.as_str(), "exe" | "msi") {
        return Err(AppError::Validation(
            "Tauri 更新包类型只允许 exe 或 msi".into(),
        ));
    }
    if query.channel != "stable" || !matches!(query.locale.as_str(), "zh-CN" | "en") {
        return Err(AppError::Validation("Tauri 更新渠道或语言无效".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::{SourceKind, UpdateAsset, UpdateSource};

    #[test]
    fn tauri_query_accepts_only_frozen_windows_targets() {
        let mut query = TauriUpdateQuery {
            architecture: "x86_64".into(),
            package_kind: "exe".into(),
            current_version: "0.7.7".into(),
            current_install_sha256: Some("a".repeat(64)),
            channel: "stable".into(),
            locale: "zh-CN".into(),
        };
        assert!(validate_query(&query).is_ok());
        let shared = shared_check_query(&query);
        assert_eq!(shared.package_kind, "exe");
        assert_eq!(shared.current_install_sha256, Some("a".repeat(64)));
        query.package_kind = "zip".into();
        assert!(validate_query(&query).is_err());
        query.package_kind = "msi".into();
        query.architecture = "aarch64".into();
        assert!(validate_query(&query).is_err());
    }

    #[test]
    fn tauri_manifest_uses_the_fixed_target_and_absolute_cloud_url() {
        let asset_id = Uuid::now_v7();
        let source_id = Uuid::now_v7();
        let response = project_response(
            LatestUpdate {
                version: "0.8.0".into(),
                channel: "stable".into(),
                title: "Creation-SSH 0.8.0".into(),
                notes: "notes".into(),
                published_at: Utc::now(),
                assets: vec![UpdateAsset {
                    id: asset_id,
                    architecture: "x86_64".into(),
                    package_kind: "exe".into(),
                    file_name: "Creation-SSH.exe".into(),
                    byte_size: 1,
                    sha256: "a".repeat(64),
                    updater_signature: Some("b".repeat(64)),
                    sources: vec![UpdateSource {
                        source_kind: SourceKind::Local,
                        provider_name: "本站".into(),
                        download_url: format!(
                            "/api/v1/downloads/assets/{asset_id}/sources/{source_id}"
                        ),
                    }],
                }],
            },
            "exe",
            "x86_64",
        )
        .expect("合法 Tauri 清单应可生成");
        let target = response
            .platforms
            .get("windows-x86_64")
            .expect("固定 target 必须存在");
        assert!(
            target
                .url
                .starts_with("https://c-ssh.com/api/v1/downloads/")
        );
        assert_eq!(target.signature, "b".repeat(64));
    }
}
