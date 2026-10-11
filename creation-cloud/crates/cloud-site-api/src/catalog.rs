//! 统一投影已发布下载清单，供旧模板与Nuxt公开页面共同使用。

use super::manual_downloads::{self, ManualNotice, ManualPlatform};
use super::release_notes::{self, NotesBlock};
use cloud_download::{PublicAsset, PublicRelease};
use cloud_site::Locale;

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublishedCatalogView {
    pub releases: Vec<PublishedReleaseView>,
    pub latest: Option<PublishedReleaseView>,
}

impl PublishedCatalogView {
    pub fn new(manifest: Vec<PublicRelease>, locale: Locale) -> Self {
        let releases = manifest
            .into_iter()
            .map(|release| PublishedReleaseView::new(release, locale))
            .collect::<Vec<_>>();
        let latest = releases.first().cloned();
        Self { releases, latest }
    }

    pub fn has_published(&self) -> bool {
        !self.releases.is_empty()
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublishedReleaseView {
    pub manual_platforms: Vec<ManualPlatform>,
    pub manual_notice: Option<ManualNotice>,
    pub version: String,
    pub revision: i32,
    pub channel: String,
    pub title: String,
    pub notes_blocks: Vec<NotesBlock>,
    pub published_at_iso: String,
    pub published_at_label: String,
    pub platforms: Vec<PublishedPlatformView>,
    pub has_windows: bool,
    pub has_linux: bool,
    pub has_android: bool,
    pub has_macos: bool,
    pub has_ios: bool,
}

impl PublishedReleaseView {
    fn new(release: PublicRelease, locale: Locale) -> Self {
        let title = match locale {
            Locale::ZhCn => release.title_zh,
            Locale::En => release.title_en,
        };
        let notes = match locale {
            Locale::ZhCn => release.notes_zh,
            Locale::En => release.notes_en,
        };
        let published_at_iso = release.published_at.to_rfc3339();
        let published_at_label = release
            .published_at
            .format("%Y-%m-%d %H:%M UTC")
            .to_string();
        let mut platforms = Vec::<PublishedPlatformView>::new();
        for asset in release.assets {
            append_asset(&mut platforms, asset);
        }
        platforms.sort_by_key(|platform| platform_rank(&platform.name));
        let has_windows = has_platform(&platforms, "windows");
        let has_linux = has_platform(&platforms, "linux");
        let has_android = has_platform(&platforms, "android");
        let has_macos = has_platform(&platforms, "macos");
        let has_ios = has_platform(&platforms, "ios");
        Self {
            manual_platforms: manual_downloads::platforms(
                &release.version,
                release.revision,
                locale,
                &platforms,
            ),
            manual_notice: manual_downloads::notice(&release.version, locale),
            version: release.version,
            revision: release.revision,
            channel: release.channel,
            title,
            notes_blocks: release_notes::parse(&notes),
            published_at_iso,
            published_at_label,
            platforms,
            has_windows,
            has_linux,
            has_android,
            has_macos,
            has_ios,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublishedPlatformView {
    pub name: String,
    pub assets: Vec<PublishedAssetView>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PublishedAssetView {
    pub architecture: String,
    pub package_kind: String,
    pub download_href: Option<String>,
}

fn append_asset(platforms: &mut Vec<PublishedPlatformView>, asset: PublicAsset) {
    let platform_name = display_platform(&asset.platform);
    let download_href = asset
        .sources
        .into_iter()
        .min_by_key(|source| source.sort_order)
        .map(|source| format!("/api/v1/downloads/{}", source.download_path));
    let asset_view = PublishedAssetView {
        architecture: asset.architecture,
        package_kind: asset.package_kind,
        download_href,
    };
    if let Some(platform) = platforms
        .iter_mut()
        .find(|platform| platform.name.eq_ignore_ascii_case(&platform_name))
    {
        platform.assets.push(asset_view);
    } else {
        platforms.push(PublishedPlatformView {
            name: platform_name,
            assets: vec![asset_view],
        });
    }
}

fn has_platform(platforms: &[PublishedPlatformView], expected: &str) -> bool {
    platforms
        .iter()
        .any(|platform| platform.name.eq_ignore_ascii_case(expected))
}

fn display_platform(platform: &str) -> String {
    match platform.to_ascii_lowercase().as_str() {
        "windows" => "Windows".to_owned(),
        "linux" => "Linux".to_owned(),
        "android" => "Android".to_owned(),
        "macos" => "macOS".to_owned(),
        "ios" => "iOS".to_owned(),
        _ => platform.to_owned(),
    }
}

fn platform_rank(platform: &str) -> usize {
    match platform.to_ascii_lowercase().as_str() {
        "windows" => 0,
        "linux" => 1,
        "android" => 2,
        "macos" => 3,
        "ios" => 4,
        _ => 5,
    }
}
