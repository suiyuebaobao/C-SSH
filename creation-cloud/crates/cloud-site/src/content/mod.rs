//! 按语言分发静态内容目录并保持模板视图结构一致。

mod en;
mod en_account;
mod en_documentation;
mod en_feedback;
mod en_home;
mod en_public;
mod en_tutorials;
mod en_workspace;
mod zh_cn;
mod zh_cn_account;
mod zh_cn_documentation;
mod zh_cn_feedback;
mod zh_cn_home;
mod zh_cn_public;
mod zh_cn_tutorials;
mod zh_cn_workspace;

use crate::{Locale, PageId, SiteView};

pub(super) fn view(page: PageId, locale: Locale) -> SiteView {
    match locale {
        Locale::ZhCn => zh_cn::view(page),
        Locale::En => en::view(page),
    }
}
