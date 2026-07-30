//! 定义官网页头公开推广仓入口，禁止混入私有源码仓地址。

const PUBLIC_REPOSITORY_URL: &str = "https://github.com/suiyuebaobao/C-SSH";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryLink {
    pub label: String,
    pub href: String,
    pub aria_label: String,
}

impl RepositoryLink {
    #[must_use]
    pub(crate) fn github(aria_label: &str) -> Self {
        Self {
            label: "GitHub".to_owned(),
            href: PUBLIC_REPOSITORY_URL.to_owned(),
            aria_label: aria_label.to_owned(),
        }
    }
}
