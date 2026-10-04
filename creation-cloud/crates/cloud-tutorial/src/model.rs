//! 定义教学视频输入、持久记录及不含文件路径的公开／管理投影。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metadata {
    pub locale: String,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    #[serde(default)]
    pub version_note: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateInput {
    pub expected_revision: i64,
    pub metadata: Metadata,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionInput {
    pub expected_revision: i64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListQuery {
    pub locale: Option<String>,
    pub offset: Option<u32>,
}

#[derive(Clone, Debug, FromRow)]
pub(crate) struct Record {
    pub id: Uuid,
    pub locale: String,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub version_note: String,
    pub state: String,
    pub revision: i64,
    pub video_bytes: i64,
    pub video_sha256: String,
    pub poster_bytes: i64,
    pub poster_sha256: String,
    pub updated_at: DateTime<Utc>,
    pub purged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PublicVideo {
    pub id: Uuid,
    pub locale: String,
    pub title: String,
    pub description: String,
    pub sort_order: i32,
    pub version_note: String,
    pub video_url: String,
    pub poster_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AdminVideo {
    #[serde(flatten)]
    pub video: PublicVideo,
    pub state: String,
    pub revision: i64,
    pub video_bytes: i64,
    pub updated_at: DateTime<Utc>,
    pub cleanup_pending: bool,
}

impl Record {
    pub fn public_view(&self) -> PublicVideo {
        PublicVideo {
            id: self.id,
            locale: self.locale.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            sort_order: self.sort_order,
            version_note: self.version_note.clone(),
            video_url: format!("/api/v1/tutorials/{}/video", self.id),
            poster_url: format!("/api/v1/tutorials/{}/poster", self.id),
        }
    }

    pub fn admin_view(&self) -> AdminVideo {
        let mut video = self.public_view();
        video.video_url = format!("/api/v1/admin/tutorials/{}/video", self.id);
        video.poster_url = format!("/api/v1/admin/tutorials/{}/poster", self.id);
        AdminVideo {
            video,
            state: self.state.clone(),
            revision: self.revision,
            video_bytes: self.video_bytes,
            updated_at: self.updated_at,
            cleanup_pending: self.state == "deleted" && self.purged_at.is_none(),
        }
    }
}
