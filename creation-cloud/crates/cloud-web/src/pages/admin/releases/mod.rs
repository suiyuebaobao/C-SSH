//! 分页展示真实发布版本及其状态机位置。
//! 创建、元数据更新、状态迁移和删除分别由独立写处理器承担。

pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod policy_publish;
pub(crate) mod policy_save;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_release::Release;
use cloud_site::{Locale, PageId, SiteView};

use crate::{AdminPageState, seo::SeoHead};

use super::shared::{self, AdminListQuery};

struct ReleaseRow {
    id: String,
    version: String,
    channel: &'static str,
    status: &'static str,
    title_zh: String,
    title_en: String,
    notes_zh: String,
    notes_en: String,
    published_at: String,
    updated_at: String,
}

struct PolicyTargetOption {
    id: String,
    label: String,
    selected: bool,
    eligible: bool,
}

struct PolicyPanel {
    draft_revision: i64,
    enabled: bool,
    forced_versions: String,
    sha256_enabled: bool,
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
    Query(query): Query<AdminListQuery>,
) -> AppResult<Html<String>> {
    let locale = query.locale();
    let actor = shared::actor_from_session(&session)?;
    let page_query = query.page_query();
    let (policy, policy_error) = match state.download().admin_update_policy(&actor).await {
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
        Ok(page) => (
            page.items.into_iter().map(ReleaseRow::from).collect(),
            page.total,
            None,
        ),
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
    let previous_href = (page_query.page > 1).then(|| release_href(page_query.page - 1, locale));
    let next_href = (i64::from(page_query.page) * i64::from(page_query.size) < total)
        .then(|| release_href(page_query.page + 1, locale));
    let parts = shared::page_parts(PageId::AdminReleases, locale, &session);
    shared::render(&ReleasesTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        rows,
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
        let selected_id = value.draft.target_release_id;
        Self {
            draft_revision: value.draft.revision,
            enabled: value.draft.enabled,
            forced_versions: value.draft.forced_versions.join("\n"),
            sha256_enabled: value.draft.sha256_enabled,
            published_revision: value.published.revision,
            targets: value
                .target_releases
                .into_iter()
                .map(|target| PolicyTargetOption {
                    id: target.id.to_string(),
                    label: format!("{} · {}", target.version, target.readiness),
                    selected: selected_id == Some(target.id),
                    eligible: target.eligible,
                })
                .collect(),
        }
    }
}

impl From<Release> for ReleaseRow {
    fn from(value: Release) -> Self {
        Self {
            id: value.id.to_string(),
            version: value.version,
            channel: value.channel.as_str(),
            status: value.status.as_str(),
            title_zh: value.title_zh,
            title_en: value.title_en,
            notes_zh: value.notes_zh,
            notes_en: value.notes_en,
            published_at: value
                .published_at
                .map_or_else(|| "—".to_owned(), |at| at.to_rfc3339()),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

fn release_href(page: u32, locale: Locale) -> String {
    shared::localized_admin_path(&format!("/admin/releases?page={page}"), locale)
}

#[cfg(test)]
mod tests {
    const TEMPLATE: &str = include_str!("../../../../templates/admin-releases.html");

    #[test]
    fn update_policy_form_keeps_exactly_the_three_business_choices() {
        for field in [
            "name=\"forced_versions\"",
            "name=\"target_release_id\"",
            "name=\"sha256_enabled\"",
            "value=\"publish_update_policy\"",
        ] {
            assert!(TEMPLATE.contains(field), "策略后台缺少字段 {field}");
        }
        assert!(!TEMPLATE.contains("name=\"download_url\""));
        assert!(!TEMPLATE.contains("name=\"updater_signature\""));
    }

    #[test]
    fn release_page_keeps_one_simple_flow_and_hides_advanced_details() {
        for marker in [
            "class=\"release-steps\"",
            "class=\"release-create-panel\"",
            "class=\"release-advanced\"",
            "class=\"release-details\"",
            "class=\"release-danger\"",
            "2. 上传三个文件",
            "3. 发布版本",
        ] {
            assert!(TEMPLATE.contains(marker), "发布页缺少渐进式入口 {marker}");
        }
        assert_eq!(TEMPLATE.matches("hx-post=\"/admin/releases\"").count(), 1);
        assert!(TEMPLATE.contains("href=\"/admin/assets?release_id={{ row.id }}"));
        assert!(TEMPLATE.find("创建版本").unwrap() < TEMPLATE.find("上传三个文件").unwrap());
        assert!(TEMPLATE.find("上传三个文件").unwrap() < TEMPLATE.find("发布并选择").unwrap());
        assert!(!TEMPLATE.contains("上传四个文件"));
    }
}
