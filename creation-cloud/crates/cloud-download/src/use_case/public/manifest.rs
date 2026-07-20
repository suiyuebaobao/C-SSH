//! 将扁平只读投影聚合为版本、资产、来源三级公开清单。

use cloud_domain::{AppError, AppResult};

use crate::{
    PublicRelease, Service,
    model::{PublicAsset, PublicCatalogRow},
    repository,
};

impl Service {
    pub async fn public_manifest(&self) -> AppResult<Vec<PublicRelease>> {
        assemble(repository::public::catalog::execute(&self.pool).await?)
    }
}

pub(crate) fn assemble(rows: Vec<PublicCatalogRow>) -> AppResult<Vec<PublicRelease>> {
    let mut releases: Vec<PublicRelease> = Vec::new();
    for row in rows {
        if releases
            .last()
            .is_none_or(|release| release.id != row.release_id)
        {
            releases.push(PublicRelease {
                id: row.release_id,
                version: row.version.clone(),
                channel: row.channel.clone(),
                title_zh: row.title_zh.clone(),
                title_en: row.title_en.clone(),
                notes_zh: row.notes_zh.clone(),
                notes_en: row.notes_en.clone(),
                published_at: row.published_at,
                assets: Vec::new(),
            });
        }
        let Some(release) = releases.last_mut() else {
            return Err(AppError::Internal("公开版本聚合状态无效".into()));
        };
        if release
            .assets
            .last()
            .is_none_or(|asset| asset.id != row.asset_id)
        {
            release.assets.push(PublicAsset {
                id: row.asset_id,
                platform: row.platform.clone(),
                architecture: row.architecture.clone(),
                package_kind: row.package_kind.clone(),
                file_name: row.file_name.clone(),
                byte_size: row.byte_size,
                sha256: row.sha256.clone(),
                sources: Vec::new(),
            });
        }
        let Some(asset) = release.assets.last_mut() else {
            return Err(AppError::Internal("公开资产聚合状态无效".into()));
        };
        asset.sources.push(row.source()?);
    }
    Ok(releases)
}
