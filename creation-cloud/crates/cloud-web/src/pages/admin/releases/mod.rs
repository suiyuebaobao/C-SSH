//! 分页展示真实发布版本及其状态机位置。
//! 创建、元数据更新、状态迁移和删除分别由独立写处理器承担。

mod catalog;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod policy_apply;
pub(crate) mod policy_publish;
pub(crate) mod policy_save;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession, normalize_semantic_version};
use cloud_site::{Locale, PageId, SiteView};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};
use catalog::ReleaseRow;

struct PolicyTargetOption {
    id: String,
    label: String,
    selected: bool,
    eligible: bool,
}

#[derive(Clone)]
struct PolicyVersionOption {
    version: String,
    mode: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct ReleasesQuery {
    #[serde(flatten)]
    list: AdminListQuery,
    tab: Option<String>,
}

struct PolicyPanel {
    draft_revision: i64,
    versions: Vec<PolicyVersionOption>,
    published_revision: i64,
    targets: Vec<PolicyTargetOption>,
}

#[derive(Template)]
#[template(path = "admin-releases.html")]
struct ReleasesTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    rows: Vec<ReleaseRow>,
    tab: String,
    policy: Option<PolicyPanel>,
    policy_error: Option<String>,
    load_error: Option<String>,
    page_number: u32,
    total: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<ReleasesQuery>,
) -> AppResult<Html<String>> {
    let locale = query.list.locale();
    let tab = match query.tab.as_deref() {
        Some("upload") => "upload",
        Some("uploaded") => "uploaded",
        _ => "settings",
    };
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.list.page_query();
    let (mut policy, policy_error) = match state.download().admin_update_policy(&actor).await {
        Ok(snapshot) => (Some(PolicyPanel::from(snapshot)), None),
        Err(_) => (
            None,
            Some(if locale == Locale::En {
                "Update policy is temporarily unavailable.".to_owned()
            } else {
                "版本策略暂时无法读取。".to_owned()
            }),
        ),
    };
    let (rows, total, load_error) = match state.release().list_releases(&actor, page_query).await {
        Ok(page) => {
            let total = page.total;
            let rows = catalog::load_rows(&state, &actor, page.items).await;
            (rows, total, None)
        }
        Err(_) => (
            Vec::new(),
            0,
            Some(if locale == Locale::En {
                "Releases are temporarily unavailable.".to_owned()
            } else {
                "版本列表暂时无法读取。".to_owned()
            }),
        ),
    };
    if let Some(policy) = policy.as_mut() {
        policy.include_releases(&rows);
    }
    let previous_href =
        (page_query.page > 1).then(|| release_href(page_query.page - 1, locale, tab));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| release_href(page_query.page + 1, locale, tab));
    let parts = shared::page_parts(PageId::AdminReleases, locale, &session);
    shared::render(&ReleasesTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
        tab: tab.into(),
        policy,
        policy_error,
        load_error,
        page_number: page_query.page,
        total,
        previous_href,
        next_href,
    })
}

impl From<cloud_download::AdminUpdatePolicySnapshot> for PolicyPanel {
    fn from(value: cloud_download::AdminUpdatePolicySnapshot) -> Self {
        let selected_id = value.published.target_release_id;
        let mut versions = value
            .target_releases
            .iter()
            .map(|target| target.version.clone())
            .chain(value.published.forced_versions.iter().cloned())
            .chain(value.published.disabled_versions.iter().cloned())
            .chain(value.published.no_update_versions.iter().cloned())
            .collect::<Vec<_>>();
        versions.sort();
        versions.dedup();
        Self {
            draft_revision: value.draft.revision,
            versions: versions
                .into_iter()
                .map(|version| {
                    let mode = if value.published.disabled_versions.contains(&version) {
                        "disabled"
                    } else if value.published.forced_versions.contains(&version) {
                        "forced"
                    } else if value.published.no_update_versions.contains(&version) {
                        "no_update"
                    } else {
                        "optional"
                    };
                    PolicyVersionOption { version, mode }
                })
                .collect(),
            published_revision: value.published.revision,
            targets: value
                .target_releases
                .into_iter()
                .map(|target| PolicyTargetOption {
                    id: target.id.to_string(),
                    label: target.version,
                    selected: selected_id == Some(target.id),
                    eligible: target.eligible,
                })
                .collect(),
        }
    }
}

impl PolicyPanel {
    fn include_releases(&mut self, rows: &[ReleaseRow]) {
        for row in rows {
            if !self
                .versions
                .iter()
                .any(|version| version.version == row.version)
            {
                self.versions.push(PolicyVersionOption {
                    version: row.version.clone(),
                    mode: "optional",
                });
            }
        }
        self.versions.sort_by(|left, right| {
            match (
                normalize_semantic_version(&left.version),
                normalize_semantic_version(&right.version),
            ) {
                (Some((_, left)), Some((_, right))) => right.cmp(&left),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => right.version.cmp(&left.version),
            }
        });
    }
}

fn release_href(page: u32, locale: Locale, tab: &str) -> String {
    shared::localized_admin_path(&format!("/admin/releases?tab={tab}&page={page}"), locale)
}
