//! 验证已发布资产巡检阶段保持既定的跨域执行顺序。

use super::*;

#[test]
fn inspection_stages_keep_download_before_site_media() {
    assert_eq!(
        INSPECTION_STAGES,
        [InspectionStage::DownloadAssets, InspectionStage::SiteMedia,]
    );
}
