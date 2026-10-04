//! 提供教学影片页面的双语页壳；实际影片只从教学领域读取。

use crate::{Locale, PageContent, PageId};

pub(super) fn page(locale: Locale) -> PageContent {
    match locale {
        Locale::ZhCn => PageContent::new(
            PageId::Tutorials,
            "C-SSH 视频教程｜主机管理、日常运维与 AI",
            "观看 C-SSH 的实际操作教学，了解 SSH、RDP、持久终端、文件、监控、AI 和云端同步。",
            "WATCH & LEARN",
            "从第一次连接，到每天都顺手",
            "通过实际客户端画面学习 C-SSH。选择一集，按自己的节奏观看。",
        ),
        Locale::En => PageContent::new(
            PageId::Tutorials,
            "C-SSH Video Tutorials | Hosts, Daily Operations and AI",
            "Learn C-SSH with recorded workflows for SSH, RDP, persistent terminals, files, monitoring, AI and cloud sync.",
            "WATCH & LEARN",
            "From your first connection to everyday work",
            "Learn through real client recordings. Choose an episode and watch at your own pace.",
        ),
    }
}
