//! 只读复核 published 站点媒体并保存观察，业务异常不改写发布状态。

use std::path::Path;

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;

use crate::{
    PublishedMediaInspectionReport, Service, file_inspection,
    inspection_model::{
        InspectedFile, PublishedMediaCandidate, SiteMediaInspectionObservation,
        SiteMediaInspectionStatus,
    },
    repository, storage,
};

impl Service {
    pub async fn inspect_published_media(&self) -> AppResult<PublishedMediaInspectionReport> {
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|_| AppError::Storage("无法取得站点媒体巡检数据库连接".into()))?;
        self.inspect_published_media_with_connection(&mut connection)
            .await
    }

    pub async fn inspect_published_media_with_connection(
        &self,
        connection: &mut PgConnection,
    ) -> AppResult<PublishedMediaInspectionReport> {
        let candidates = repository::inspection::list_published(connection).await?;
        let mut observations = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            observations.push(inspect_candidate(self.site_media_root.as_path(), candidate).await);
        }
        repository::inspection::save_observations(connection, &observations).await?;
        Ok(PublishedMediaInspectionReport {
            inspected_media: observations.len() as u64,
            finding_count: observations
                .iter()
                .filter(|observation| observation.finding())
                .count() as u64,
        })
    }
}

pub(crate) async fn inspect_candidate(
    root: &Path,
    candidate: PublishedMediaCandidate,
) -> SiteMediaInspectionObservation {
    let media_id = candidate.media_id;
    let file = match storage::resolve_existing_for_inspection(root, &candidate.storage_key).await {
        Ok(path) => {
            file_inspection::inspect(
                &path,
                candidate.expected_byte_size,
                &candidate.expected_sha256,
            )
            .await
        }
        Err(AppError::NotFound(_)) => InspectedFile {
            status: SiteMediaInspectionStatus::Missing,
            observed_byte_size: None,
            observed_sha256: None,
        },
        Err(_) => InspectedFile {
            status: SiteMediaInspectionStatus::IoError,
            observed_byte_size: None,
            observed_sha256: None,
        },
    };
    SiteMediaInspectionObservation {
        media_id,
        status: file.status,
        observed_byte_size: file.observed_byte_size,
        observed_sha256: file.observed_sha256,
    }
}
