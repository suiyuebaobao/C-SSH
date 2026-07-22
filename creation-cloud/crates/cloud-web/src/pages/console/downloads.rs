//! 渲染真实已发布清单，并诚实隔离尚未提供的账号下载历史。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_download::{DownloadHistoryItem, PublicRelease};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-downloads.html")]
struct DownloadsTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    releases: Vec<PublicRelease>,
    history: Vec<DownloadHistoryItem>,
    history_total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let (releases, history) = tokio::try_join!(
        state.download().public_manifest(),
        state
            .download()
            .account_history(&session, common::first_page()),
    )?;
    let locale = query.locale();
    common::render(&DownloadsTemplate {
        view: common::view(PageId::ConsoleDownloads, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        releases,
        history: history.items,
        history_total: history.total,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use cloud_download::{PublicAsset, PublicSource, SourceKind};
    use cloud_site::Locale;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn template_uses_account_attributed_download_route_and_real_history() {
        let asset_id = Uuid::from_u128(10);
        let source_id = Uuid::from_u128(11);
        let released_at = Utc.with_ymd_and_hms(2026, 7, 21, 8, 0, 0).unwrap();
        let releases = vec![PublicRelease {
            id: Uuid::from_u128(9),
            version: "v1.2.3".to_owned(),
            channel: "stable".to_owned(),
            title_zh: "稳定版本".to_owned(),
            title_en: "Stable release".to_owned(),
            notes_zh: "版本说明".to_owned(),
            notes_en: "Release notes".to_owned(),
            published_at: released_at,
            assets: vec![PublicAsset {
                id: asset_id,
                platform: "windows".to_owned(),
                architecture: "x86_64".to_owned(),
                package_kind: "msi".to_owned(),
                file_name: "creation-ssh-example.msi".to_owned(),
                byte_size: 123,
                sha256: "example-sha256".to_owned(),
                sources: vec![PublicSource {
                    id: source_id,
                    source_kind: SourceKind::Local,
                    provider_name: "本站".to_owned(),
                    sort_order: 0,
                    download_path: "/forbidden-public-download-path".to_owned(),
                }],
            }],
        }];
        let history = vec![DownloadHistoryItem {
            id: Uuid::from_u128(12),
            asset_id,
            source_id,
            version: "v1.2.3".to_owned(),
            platform: "windows".to_owned(),
            architecture: "x86_64".to_owned(),
            package_kind: "msi".to_owned(),
            file_name: "history-entry-example.msi".to_owned(),
            provider_name: "本站".to_owned(),
            source_kind: "local".to_owned(),
            occurred_at: released_at,
        }];

        let rendered = DownloadsTemplate {
            view: common::view(PageId::ConsoleDownloads, Locale::ZhCn),
            seo: common::seo(),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            releases,
            history,
            history_total: 1,
        }
        .render()
        .expect("下载控制台模板应可渲染");

        let expected_path =
            format!("/api/v1/downloads/account/assets/{asset_id}/sources/{source_id}");
        assert!(rendered.contains(&expected_path));
        assert!(rendered.contains("history-entry-example.msi"));
        assert!(!rendered.contains("/forbidden-public-download-path"));
    }
}
