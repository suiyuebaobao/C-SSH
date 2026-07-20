//! 通过稳定入口按页面和语言组装完整的网站视图。

use crate::{Locale, PageId, SiteView, content};

#[derive(Clone, Copy, Debug, Default)]
pub struct ContentService;

impl ContentService {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn view(self, page: PageId, locale: Locale) -> SiteView {
        content::view(page, locale)
    }
}

#[must_use]
pub const fn content_service() -> ContentService {
    ContentService::new()
}
