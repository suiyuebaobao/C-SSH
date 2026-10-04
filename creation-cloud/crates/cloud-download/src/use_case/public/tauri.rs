//! 从同一策略检查结果投影 Windows Tauri清单，确保版本、资产和revision不分叉。

use std::collections::BTreeMap;

use cloud_domain::{AppError, AppResult};
use url::Url;

use crate::{
    LatestUpdate, Service, UpdateCheckQuery,
    model::{TauriPlatformUpdate, TauriUpdateQuery, TauriUpdateResponse},
};

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
        project_response(
            latest,
            &query.package_kind,
            &query.architecture,
            &self.public_base_url,
        )
        .map(Some)
    }
}

fn shared_check_query(query: &TauriUpdateQuery) -> UpdateCheckQuery {
    UpdateCheckQuery {
        policy_version: None,
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
    public_base_url: &Url,
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
            url: public_base_url
                .join(&source.download_url)
                .map_err(|_| AppError::Storage("Tauri 下载地址无法拼接".into()))?
                .to_string(),
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
