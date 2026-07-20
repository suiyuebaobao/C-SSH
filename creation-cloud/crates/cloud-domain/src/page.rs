//! 提供所有列表接口共用的受限分页对象。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_size")]
    pub size: u32,
}

impl PageQuery {
    #[must_use]
    pub fn normalized(self) -> Self {
        Self {
            page: self.page.max(1),
            size: self.size.clamp(1, 100),
        }
    }

    #[must_use]
    pub fn offset(self) -> i64 {
        let value = self.normalized();
        i64::from((value.page - 1) * value.size)
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub total: i64,
}

const fn default_page() -> u32 {
    1
}

const fn default_size() -> u32 {
    20
}

impl Default for PageQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            size: default_size(),
        }
    }
}
