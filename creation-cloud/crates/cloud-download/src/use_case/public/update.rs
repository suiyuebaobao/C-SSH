//! 从既有公开清单筛选目标平台的最新语义版本。

use cloud_domain::{AppError, AppResult, SemanticVersion, normalize_semantic_version};

use crate::{
    LatestUpdate, PublicRelease, Service, UpdateAsset, UpdateCheckQuery, UpdateCheckResponse,
    UpdateSource,
};

const DOWNLOAD_PREFIX: &str = "/api/v1/downloads";
impl Service {
    pub async fn check_update(&self, query: UpdateCheckQuery) -> AppResult<UpdateCheckResponse> {
        let query = ValidatedQuery::try_from(query)?;
        evaluate(&self.public_manifest().await?, &query)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Locale {
    ZhCn,
    En,
}

#[derive(Debug)]
struct ValidatedQuery {
    platform: String,
    architecture: String,
    current_text: String,
    current_version: SemanticVersion,
    channel: String,
    locale: Locale,
}

impl TryFrom<UpdateCheckQuery> for ValidatedQuery {
    type Error = AppError;

    fn try_from(query: UpdateCheckQuery) -> AppResult<Self> {
        let platform = match query.platform.as_str() {
            "windows" | "linux" | "android" => query.platform,
            _ => {
                return Err(AppError::Validation(
                    "platform 必须是 windows、linux 或 android".into(),
                ));
            }
        };
        let architecture = match query.architecture.as_str() {
            "x86_64" | "aarch64" => query.architecture,
            _ => {
                return Err(AppError::Validation(
                    "architecture 必须是 x86_64 或 aarch64".into(),
                ));
            }
        };
        let channel = match query.channel.as_str() {
            "stable" | "beta" | "nightly" => query.channel,
            _ => {
                return Err(AppError::Validation(
                    "channel 必须是 stable、beta 或 nightly".into(),
                ));
            }
        };
        let locale = match query.locale.as_str() {
            "zh-CN" => Locale::ZhCn,
            "en" => Locale::En,
            _ => {
                return Err(AppError::Validation("locale 必须是 zh-CN 或 en".into()));
            }
        };
        let (current_text, current_version) = normalize_semantic_version(&query.current_version)
            .ok_or_else(|| AppError::Validation("current_version 必须是有效语义版本".into()))?;
        Ok(Self {
            platform,
            architecture,
            current_text,
            current_version,
            channel,
            locale,
        })
    }
}

fn evaluate(releases: &[PublicRelease], query: &ValidatedQuery) -> AppResult<UpdateCheckResponse> {
    let mut latest: Option<(&PublicRelease, String, SemanticVersion)> = None;
    for release in releases
        .iter()
        .filter(|release| release.channel == query.channel)
        .filter(|release| {
            release.assets.iter().any(|asset| {
                asset.platform == query.platform
                    && asset.architecture == query.architecture
                    && !asset.sources.is_empty()
            })
        })
    {
        let Some((normalized_version, version)) = normalize_semantic_version(&release.version)
        else {
            continue;
        };
        if latest
            .as_ref()
            .is_none_or(|(_, _, latest_version)| version > *latest_version)
        {
            latest = Some((release, normalized_version, version));
        }
    }

    let Some((release, normalized_version, latest_version)) = latest else {
        return Ok(no_update(query));
    };
    if latest_version <= query.current_version {
        return Ok(no_update(query));
    }

    let (title, notes) = match query.locale {
        Locale::ZhCn => (&release.title_zh, &release.notes_zh),
        Locale::En => (&release.title_en, &release.notes_en),
    };
    let assets = release
        .assets
        .iter()
        .filter(|asset| {
            asset.platform == query.platform
                && asset.architecture == query.architecture
                && !asset.sources.is_empty()
        })
        .map(|asset| UpdateAsset {
            id: asset.id,
            architecture: asset.architecture.clone(),
            package_kind: asset.package_kind.clone(),
            file_name: asset.file_name.clone(),
            byte_size: asset.byte_size,
            sha256: asset.sha256.clone(),
            sources: asset
                .sources
                .iter()
                .map(|source| UpdateSource {
                    source_kind: source.source_kind,
                    provider_name: source.provider_name.clone(),
                    download_url: format!(
                        "{DOWNLOAD_PREFIX}/assets/{}/sources/{}",
                        asset.id, source.id
                    ),
                })
                .collect(),
        })
        .collect();
    Ok(UpdateCheckResponse {
        update_available: true,
        current_version: query.current_text.clone(),
        latest: Some(LatestUpdate {
            version: normalized_version,
            channel: release.channel.clone(),
            title: title.clone(),
            notes: notes.clone(),
            published_at: release.published_at,
            assets,
        }),
    })
}

fn no_update(query: &ValidatedQuery) -> UpdateCheckResponse {
    UpdateCheckResponse {
        update_available: false,
        current_version: query.current_text.clone(),
        latest: None,
    }
}
