//! 并发复核已发布本站来源并保存只读观察，不探测任何外部 URL。

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use cloud_domain::{AppError, AppResult};
use sqlx::PgConnection;
use tokio::task::JoinSet;

use crate::{
    PublishedAssetInspectionReport, Service,
    file_verification::{InspectionVerifier, MAX_PARALLEL_HASHES},
    local_file,
    model::{
        AssetInspectionObservation, AssetInspectionStatus, FileInspection, PublishedLocalAsset,
    },
    repository,
};

impl Service {
    pub async fn inspect_published_local_assets(
        &self,
    ) -> AppResult<PublishedAssetInspectionReport> {
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(repository::map_transaction_error)?;
        self.inspect_published_local_assets_with_connection(&mut connection)
            .await
    }

    pub async fn inspect_published_local_assets_with_connection(
        &self,
        connection: &mut PgConnection,
    ) -> AppResult<PublishedAssetInspectionReport> {
        let candidates = repository::inspection::list_published_local(connection).await?;
        let verifier = InspectionVerifier::default();
        let mut remaining = candidates.into_iter();
        let mut tasks = JoinSet::new();
        for _ in 0..MAX_PARALLEL_HASHES {
            let Some(candidate) = remaining.next() else {
                break;
            };
            spawn_inspection(
                &mut tasks,
                self.download_root.clone(),
                verifier.clone(),
                candidate,
            );
        }

        let mut observations = Vec::new();
        while let Some(joined) = tasks.join_next().await {
            let observation =
                joined.map_err(|_| AppError::Internal("发布资产巡检子任务异常退出".into()))?;
            observations.push(observation);
            if let Some(candidate) = remaining.next() {
                spawn_inspection(
                    &mut tasks,
                    self.download_root.clone(),
                    verifier.clone(),
                    candidate,
                );
            }
        }
        observations.sort_by_key(|observation| observation.source_id);
        repository::inspection::save_observations(connection, &observations).await?;
        Ok(PublishedAssetInspectionReport {
            inspected_sources: observations.len() as u64,
            finding_count: observations
                .iter()
                .filter(|observation| observation.finding())
                .count() as u64,
        })
    }
}

pub(crate) async fn inspect_candidate(
    root: &Path,
    verifier: &InspectionVerifier,
    candidate: PublishedLocalAsset,
) -> AssetInspectionObservation {
    let source_id = candidate.source_id;
    let file = match local_file::resolve(root, &candidate.local_path).await {
        Ok(path) => {
            verifier
                .inspect(
                    &path,
                    candidate.expected_byte_size,
                    &candidate.expected_sha256,
                )
                .await
        }
        Err(AppError::NotFound(_)) => FileInspection {
            status: AssetInspectionStatus::Missing,
            observed_byte_size: None,
            observed_sha256: None,
        },
        Err(_) => FileInspection {
            status: AssetInspectionStatus::IoError,
            observed_byte_size: None,
            observed_sha256: None,
        },
    };
    AssetInspectionObservation {
        source_id,
        status: file.status,
        observed_byte_size: file.observed_byte_size,
        observed_sha256: file.observed_sha256,
    }
}

fn spawn_inspection(
    tasks: &mut JoinSet<AssetInspectionObservation>,
    root: Arc<PathBuf>,
    verifier: InspectionVerifier,
    candidate: PublishedLocalAsset,
) {
    tasks.spawn(async move { inspect_candidate(root.as_path(), &verifier, candidate).await });
}
