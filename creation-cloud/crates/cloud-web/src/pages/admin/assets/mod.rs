//! 分页展示安装资产身份，并聚合每个资产的真实下载来源。
//! 资产、来源与流式上传写操作分别由独立处理器承担。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod source_create;
pub(crate) mod source_delete;
pub(crate) mod source_update;
pub(crate) mod update;
pub(crate) mod upload;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession, PageQuery};
use cloud_download::ReleaseSource;
use cloud_release::{Release, ReleaseAsset};
use cloud_site::{Locale, PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct SourceRow {
    id: String,
    kind: &'static str,
    provider: String,
    location: String,
    sort_order: i32,
    enabled: bool,
}

struct AssetRow {
    id: String,
    release_id: String,
    platform: String,
    architecture: String,
    package_kind: String,
    file_name: String,
    byte_size: i64,
    sha256: String,
    sources: Vec<SourceRow>,
    source_error: bool,
    release_state_error: bool,
    identity_mutable: bool,
    source_mutable: bool,
}

struct ReleaseOption {
    id: String,
    label: String,
}

#[derive(Template)]
#[template(path = "admin-assets.html")]
struct AssetsTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<AssetRow>,
    release_options: Vec<ReleaseOption>,
    release_options_error: bool,
    load_error: Option<String>,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.page_query();
    let (release_options, release_options_error) = load_releases(&state, &actor).await;
    let (rows, total, load_error) = match state.release().list_all_assets(&actor, page_query).await
    {
        Ok(page) => {
            let mut rows = Vec::with_capacity(page.items.len());
            for asset in page.items {
                let sources = state.download().list_sources(&actor, asset.id).await;
                let release = state.release().get_release(&actor, asset.release_id).await;
                rows.push(AssetRow::new(asset, sources, release));
            }
            (rows, page.total, None)
        }
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Assets are temporarily unavailable.".to_owned()
            } else {
                "资产列表暂时无法读取。".to_owned()
            }),
        ),
    };
    let previous_href = (page_query.page > 1).then(|| asset_href(page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| asset_href(page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminAssets, locale, &session);
    shared::render(&AssetsTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        release_options,
        release_options_error,
        load_error,
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl AssetRow {
    fn new(
        asset: ReleaseAsset,
        sources: AppResult<Vec<ReleaseSource>>,
        release: AppResult<Release>,
    ) -> Self {
        let (sources, source_error) = match sources {
            Ok(items) => (items.into_iter().map(SourceRow::from).collect(), false),
            Err(_) => (Vec::new(), true),
        };
        let (release_state_error, identity_mutable, source_mutable) = match release {
            Ok(release) => (
                false,
                release.status.allows_asset_mutation(),
                !matches!(
                    release.status,
                    cloud_release::ReleaseStatus::Revoked | cloud_release::ReleaseStatus::Hidden
                ),
            ),
            Err(_) => (true, false, false),
        };
        Self {
            id: asset.id.to_string(),
            release_id: asset.release_id.to_string(),
            platform: asset.platform,
            architecture: asset.architecture,
            package_kind: asset.package_kind,
            file_name: asset.file_name,
            byte_size: asset.byte_size,
            sha256: asset.sha256,
            sources,
            source_error,
            release_state_error,
            identity_mutable,
            source_mutable,
        }
    }
}

impl From<ReleaseSource> for SourceRow {
    fn from(value: ReleaseSource) -> Self {
        Self {
            id: value.id.to_string(),
            kind: value.source_kind.as_str(),
            provider: value.provider_name,
            location: value
                .external_url
                .or(value.local_path)
                .unwrap_or_else(|| "—".to_owned()),
            sort_order: value.sort_order,
            enabled: value.enabled,
        }
    }
}

async fn load_releases(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
) -> (Vec<ReleaseOption>, bool) {
    match state
        .release()
        .list_releases(actor, PageQuery { page: 1, size: 100 })
        .await
    {
        Ok(page) => (
            page.items
                .into_iter()
                .filter(|release| release.status.allows_asset_mutation())
                .map(ReleaseOption::from)
                .collect(),
            false,
        ),
        Err(_) => (Vec::new(), true),
    }
}

impl From<Release> for ReleaseOption {
    fn from(value: Release) -> Self {
        Self {
            id: value.id.to_string(),
            label: format!(
                "{} · {} · {}",
                value.version,
                value.channel.as_str(),
                value.status.as_str()
            ),
        }
    }
}

fn asset_href(page: u32, locale: Locale) -> String {
    shared::localized_admin_path(&format!("/admin/assets?page={page}"), locale)
}
