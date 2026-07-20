//! 定义官网页头公开推广仓入口，禁止混入私有源码仓地址。

const PUBLIC_REPOSITORY_URL: &str = "https://github.com/suiyuebaobao/C-SSH";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepositoryLink {
    pub label: &'static str,
    pub href: &'static str,
    pub aria_label: &'static str,
}

impl RepositoryLink {
    #[must_use]
    pub(crate) const fn github(aria_label: &'static str) -> Self {
        Self {
            label: "GitHub",
            href: PUBLIC_REPOSITORY_URL,
            aria_label,
        }
    }
}
