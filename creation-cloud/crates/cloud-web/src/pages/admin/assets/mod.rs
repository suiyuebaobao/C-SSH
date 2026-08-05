//! 用一个极简入口新增本站或外链下载，并以常规列表展示可管理状态。
//! 文件身份仍由服务端计算，资产与来源约束继续下沉到领域用例。

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

struct AssetRow {
    asset_id: String,
    source_id: Option<String>,
    release_version: String,
    release_status: &'static str,
    platform: String,
    architecture: String,
    package_kind: String,
    file_name: String,
    source_kind: &'static str,
    source_provider: String,
    source_location: String,
    source_enabled: bool,
    has_source: bool,
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
                rows.extend(AssetRow::new(asset, sources, release));
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
    ) -> Vec<Self> {
        let (
            release_version,
            release_status,
            release_state_error,
            identity_mutable,
            source_mutable,
        ) = match release {
            Ok(release) => (
                release.version,
                release.status.as_str(),
                false,
                release.status.allows_asset_mutation(),
                !matches!(
                    release.status,
                    cloud_release::ReleaseStatus::Revoked | cloud_release::ReleaseStatus::Hidden
                ),
            ),
            Err(_) => ("—".to_owned(), "unknown", true, false, false),
        };
        let base = |source: Option<ReleaseSource>, source_error| {
            let (
                source_id,
                source_kind,
                source_provider,
                source_location,
                source_enabled,
                has_source,
            ) = match source {
                Some(source) => (
                    Some(source.id.to_string()),
                    source.source_kind.as_str(),
                    source.provider_name,
                    source
                        .external_url
                        .or(source.local_path)
                        .unwrap_or_else(|| "—".to_owned()),
                    source.enabled,
                    true,
                ),
                None => (None, "none", "—".to_owned(), "—".to_owned(), false, false),
            };
            Self {
                asset_id: asset.id.to_string(),
                source_id,
                release_version: release_version.clone(),
                release_status,
                platform: asset.platform.clone(),
                architecture: asset.architecture.clone(),
                package_kind: asset.package_kind.clone(),
                file_name: asset.file_name.clone(),
                source_kind,
                source_provider,
                source_location,
                source_enabled,
                has_source,
                source_error,
                release_state_error,
                identity_mutable,
                source_mutable,
            }
        };
        match sources {
            Ok(items) if !items.is_empty() => items
                .into_iter()
                .map(|source| base(Some(source), false))
                .collect(),
            Ok(_) => vec![base(None, false)],
            Err(_) => vec![base(None, true)],
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
                .filter(|release| {
                    !matches!(
                        release.status,
                        cloud_release::ReleaseStatus::Revoked
                            | cloud_release::ReleaseStatus::Hidden
                    )
                })
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

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_site::content_service;

    #[test]
    fn template_keeps_download_management_simple_and_strongly_validated() {
        let asset_id = "01917f21-9f82-7ca4-b1dd-034518738965";
        let source_id = "01917f21-9f82-7ca4-b1dd-034518738966";
        let body = AssetsTemplate {
            view: content_service().view(PageId::AdminAssets, Locale::ZhCn),
            seo: SeoHead::private(),
            session_identity: Some("ops-admin".to_owned()),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            rows: vec![AssetRow {
                asset_id: asset_id.to_owned(),
                source_id: Some(source_id.to_owned()),
                release_version: "7.0.0".to_owned(),
                release_status: "validating",
                platform: "windows".to_owned(),
                architecture: "x86_64".to_owned(),
                package_kind: "exe".to_owned(),
                file_name: "client<script>.exe".to_owned(),
                source_kind: "external",
                source_provider: "GitHub Release".to_owned(),
                source_location: "https://example.com/client.exe".to_owned(),
                source_enabled: true,
                has_source: true,
                source_error: false,
                release_state_error: false,
                identity_mutable: true,
                source_mutable: true,
            }],
            release_options: vec![ReleaseOption {
                id: "01917f21-9f82-7ca4-b1dd-034518738967".to_owned(),
                label: "7.0.0 · stable · validating".to_owned(),
            }],
            release_options_error: false,
            load_error: None,
            page_number: 1,
            total: 1,
            previous_href: None,
            next_href: None,
        }
        .render()
        .expect("下载管理模板应可渲染");

        assert!(body.contains("data-download-create"));
        assert!(body.contains("hx-encoding=\"multipart/form-data\""));
        assert!(body.contains("name=\"release_id\""));
        assert!(body.contains("name=\"source_mode\" value=\"local\" checked"));
        assert!(body.contains("name=\"source_mode\" value=\"external\""));
        assert!(body.contains("name=\"file\""));
        assert!(body.contains("name=\"external_url\""));
        assert!(body.contains("value=\"macos\" disabled"));
        assert!(body.contains("value=\"ios\" disabled"));
        assert!(body.contains("data-platform=\"android\">APK"));
        assert!(!body.contains(">AAB<"));
        assert!(body.contains("data-download-validation"));
        assert_eq!(body.matches("<table").count(), 1);
        assert!(body.contains("name=\"byte_size\""));
        assert!(body.contains("name=\"sha256\""));
        assert!(!body.contains("name=\"provider_name\""));
        assert!(!body.contains("name=\"sort_order\""));
        assert!(body.contains(&format!("hx-post=\"/admin/assets/{asset_id}\"")));
        assert!(body.contains(&format!("hx-post=\"/admin/sources/{source_id}\"")));
        assert!(body.contains(&format!("hx-post=\"/admin/sources/{source_id}/delete\"")));
        assert!(!body.contains("client<script>.exe"));
    }
}
