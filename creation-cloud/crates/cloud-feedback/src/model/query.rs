//! 定义管理反馈状态筛选与显式分页 wire，拒绝未知查询字段。

use cloud_domain::PageQuery;
use serde::{Deserialize, Deserializer};

use super::FeedbackStatus;

#[derive(Clone, Copy, Debug)]
pub struct AdminFeedbackListQuery {
    pub page: PageQuery,
    pub status: Option<FeedbackStatus>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminFeedbackListQueryWire {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_size")]
    size: u32,
    status: Option<FeedbackStatus>,
}

impl<'de> Deserialize<'de> for AdminFeedbackListQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AdminFeedbackListQueryWire::deserialize(deserializer)?;
        Ok(Self {
            page: PageQuery {
                page: wire.page,
                size: wire.size,
            },
            status: wire.status,
        })
    }
}

const fn default_page() -> u32 {
    1
}

const fn default_size() -> u32 {
    20
}
