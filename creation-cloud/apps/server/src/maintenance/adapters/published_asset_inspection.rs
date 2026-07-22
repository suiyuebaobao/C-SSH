//! 固定顺序调用下载与站点媒体只读巡检，限制组合任务的文件哈希并发上界。

use cloud_domain::AppResult;
use cloud_maintenance::{ObservationCode, TaskExecutionReport};
use sqlx::PgConnection;

use crate::{maintenance::progress::CommittedProgress, services::AppServices};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InspectionStage {
    DownloadAssets,
    SiteMedia,
}

const INSPECTION_STAGES: [InspectionStage; 2] =
    [InspectionStage::DownloadAssets, InspectionStage::SiteMedia];

pub async fn execute(
    services: &AppServices,
    connection: &mut PgConnection,
    progress: &CommittedProgress,
) -> AppResult<TaskExecutionReport> {
    // 实际执行与顺序测试共同遍历此表，确保两个域的哈希工作不会重叠。
    for stage in INSPECTION_STAGES {
        match stage {
            InspectionStage::DownloadAssets => {
                let report = services
                    .download
                    .inspect_published_local_assets_with_connection(&mut *connection)
                    .await?;
                progress.add(TaskExecutionReport {
                    examined_count: report.inspected_sources,
                    healthy_count: report
                        .inspected_sources
                        .saturating_sub(report.finding_count),
                    issue_count: report.finding_count,
                    ..TaskExecutionReport::default()
                });
            }
            InspectionStage::SiteMedia => {
                let report = services
                    .site_media
                    .inspect_published_media_with_connection(&mut *connection)
                    .await?;
                progress.add(TaskExecutionReport {
                    examined_count: report.inspected_media,
                    healthy_count: report.inspected_media.saturating_sub(report.finding_count),
                    issue_count: report.finding_count,
                    ..TaskExecutionReport::default()
                });
            }
        }
    }
    let mut report = progress.snapshot();
    report.observation = Some(if report.issue_count == 0 {
        ObservationCode::Healthy
    } else {
        ObservationCode::IssuesDetected
    });
    Ok(report)
}

#[cfg(test)]
#[path = "published_asset_inspection_tests.rs"]
mod tests;
