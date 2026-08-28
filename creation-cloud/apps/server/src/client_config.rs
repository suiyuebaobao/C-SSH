//! 为正式客户端聚合一次启动所需的公开事实，不建立第二套公告、版本或登录设置数据。

use axum::{
    Json, Router,
    extract::{RawQuery, State},
    routing::get,
};
use chrono::{DateTime, Utc};
use cloud_announcement::{AnnouncementLocale, AnnouncementPriority};
use cloud_domain::{AppError, AppResult, normalize_semantic_version};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClientConfigState {
    announcement: cloud_announcement::Service,
    auth: cloud_auth::Service,
    download: cloud_download::Service,
}

impl ClientConfigState {
    #[must_use]
    pub fn new(
        announcement: cloud_announcement::Service,
        auth: cloud_auth::Service,
        download: cloud_download::Service,
    ) -> Self {
        Self {
            announcement,
            auth,
            download,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClientConfigResponse {
    pub schema_version: u8,
    pub announcement: ClientAnnouncement,
    pub latest_version: Option<ClientLatestVersion>,
    pub version_policy: cloud_download::PublishedUpdatePolicy,
    pub login: cloud_auth::ClientLoginConfig,
}

#[derive(Debug, Serialize)]
pub struct ClientAnnouncement {
    pub revision: i64,
    pub id: Option<Uuid>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub priority: Option<AnnouncementPriority>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ClientLatestVersion {
    pub version: String,
    pub channel: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<ClientVersionAsset>,
}

#[derive(Debug, Serialize)]
pub struct ClientVersionAsset {
    pub id: Uuid,
    pub platform: String,
    pub architecture: String,
    pub package_kind: String,
    pub file_name: String,
    pub byte_size: i64,
    pub sha256: String,
    pub updater_signature: Option<String>,
    pub sources: Vec<ClientDownloadSource>,
}

#[derive(Debug, Serialize)]
pub struct ClientDownloadSource {
    pub id: Uuid,
    pub source_kind: cloud_download::SourceKind,
    pub provider_name: String,
    pub sort_order: i32,
    pub download_path: String,
}

#[must_use = "the client config router must be mounted to become reachable"]
pub fn router(state: ClientConfigState) -> Router {
    Router::new()
        .route("/config", get(handle))
        .with_state(state)
}

async fn handle(
    RawQuery(query): RawQuery,
    State(state): State<ClientConfigState>,
) -> AppResult<Json<ClientConfigResponse>> {
    if query.is_some() {
        return Err(AppError::Validation(
            "客户端配置接口不接受查询参数".to_owned(),
        ));
    }
    let (announcement, manifest, version_policy, login) = tokio::try_join!(
        state.announcement.current(AnnouncementLocale::ZhCn),
        state.download.public_manifest(),
        state.download.public_update_policy(),
        state.auth.client_login_config(),
    )?;
    let announcement = ClientAnnouncement::from(announcement);
    let latest_version = latest_stable_release(manifest).map(ClientLatestVersion::from);
    Ok(Json(ClientConfigResponse {
        schema_version: 2,
        announcement,
        latest_version,
        version_policy,
        login,
    }))
}

fn latest_stable_release(
    releases: Vec<cloud_download::PublicRelease>,
) -> Option<cloud_download::PublicRelease> {
    releases
        .into_iter()
        .filter(|release| release.channel == "stable")
        .filter_map(|release| {
            normalize_semantic_version(&release.version).map(|(_, version)| (version, release))
        })
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, release)| release)
}

impl From<cloud_announcement::CurrentAnnouncementResponse> for ClientAnnouncement {
    fn from(value: cloud_announcement::CurrentAnnouncementResponse) -> Self {
        match value.announcement {
            Some(announcement) => Self {
                revision: value.revision,
                id: Some(announcement.id),
                title: Some(announcement.title),
                body: Some(announcement.content),
                priority: Some(announcement.priority),
                published_at: Some(announcement.published_at),
            },
            None => Self {
                revision: value.revision,
                id: None,
                title: None,
                body: None,
                priority: None,
                published_at: None,
            },
        }
    }
}

impl From<cloud_download::PublicRelease> for ClientLatestVersion {
    fn from(value: cloud_download::PublicRelease) -> Self {
        Self {
            version: value.version,
            channel: value.channel,
            published_at: value.published_at,
            assets: value
                .assets
                .into_iter()
                .map(ClientVersionAsset::from)
                .collect(),
        }
    }
}

impl From<cloud_download::PublicAsset> for ClientVersionAsset {
    fn from(value: cloud_download::PublicAsset) -> Self {
        Self {
            id: value.id,
            platform: value.platform,
            architecture: value.architecture,
            package_kind: value.package_kind,
            file_name: value.file_name,
            byte_size: value.byte_size,
            sha256: value.sha256,
            updater_signature: value.updater_signature,
            sources: value
                .sources
                .into_iter()
                .map(ClientDownloadSource::from)
                .collect(),
        }
    }
}

impl From<cloud_download::PublicSource> for ClientDownloadSource {
    fn from(value: cloud_download::PublicSource) -> Self {
        Self {
            id: value.id,
            source_kind: value.source_kind,
            provider_name: value.provider_name,
            sort_order: value.sort_order,
            download_path: value.download_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use cloud_announcement::PublicAnnouncement;

    use super::*;

    #[test]
    fn maps_the_single_published_announcement_without_language_metadata() {
        let id = Uuid::now_v7();
        let published_at = Utc::now();
        let mapped = ClientAnnouncement::from(cloud_announcement::CurrentAnnouncementResponse {
            revision: 7,
            announcement: Some(PublicAnnouncement {
                id,
                title: "公告".to_owned(),
                content: "正文".to_owned(),
                priority: AnnouncementPriority::Important,
                published_at,
                updated_at: published_at,
            }),
        });
        assert_eq!(mapped.revision, 7);
        assert_eq!(mapped.id, Some(id));
        assert_eq!(mapped.body.as_deref(), Some("正文"));
        assert_eq!(mapped.priority, Some(AnnouncementPriority::Important));
    }

    #[test]
    fn response_keeps_the_five_field_startup_contract() {
        let value = serde_json::to_value(ClientConfigResponse {
            schema_version: 2,
            announcement: ClientAnnouncement {
                revision: 1,
                id: None,
                title: None,
                body: None,
                priority: None,
                published_at: None,
            },
            latest_version: None,
            version_policy: cloud_download::PublishedUpdatePolicy::disabled(),
            login: cloud_auth::ClientLoginConfig {
                revision: 1,
                captcha_enabled: true,
                email_code_enabled: true,
            },
        })
        .expect("客户端配置响应应可序列化");
        let object = value.as_object().expect("客户端配置响应应为对象");
        assert_eq!(object.len(), 5);
        for key in [
            "schema_version",
            "announcement",
            "latest_version",
            "version_policy",
            "login",
        ] {
            assert!(object.contains_key(key), "缺少字段 {key}");
        }
        assert!(!value.to_string().contains("locale"));
        assert!(!value.to_string().contains("language"));
    }

    #[test]
    fn latest_version_is_the_semantic_maximum_published_stable_release() {
        let release = |version: &str, channel: &str| cloud_download::PublicRelease {
            id: Uuid::now_v7(),
            version: version.to_owned(),
            channel: channel.to_owned(),
            title_zh: "标题".to_owned(),
            title_en: "Title".to_owned(),
            notes_zh: "说明".to_owned(),
            notes_en: "Notes".to_owned(),
            published_at: Utc::now(),
            assets: Vec::new(),
        };
        let latest = latest_stable_release(vec![
            release("0.8.0-beta.1", "beta"),
            release("0.7.10", "stable"),
            release("0.7.9", "stable"),
            release("legacy", "stable"),
        ])
        .expect("应选出合法 stable 版本");
        assert_eq!(latest.version, "0.7.10");
    }
}
