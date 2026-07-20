//! 解析页面语言查询参数，不承载账号或业务筛选条件。

use cloud_site::Locale;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct LocaleQuery {
    lang: Option<String>,
}

impl LocaleQuery {
    pub(crate) fn locale(&self) -> Locale {
        Locale::from_code(self.lang.as_deref())
    }
}
